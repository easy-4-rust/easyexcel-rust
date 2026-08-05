//! Shared String Table (SST) parsing and building.
//!
//! The SST record holds many `XLUnicodeRichExtendedString`s back-to-back. Each
//! string begins with a 2-byte char count and a 1-byte grbit flag:
//!   bit0 (0x01) fHighByte: 0 => compressed (1 byte/char latin1), 1 => UTF-16LE
//!   bit2 (0x04) fExtSt:    extended (phonetic) data present (cchExtRst u32)
//!   bit3 (0x08) fRichSt:   rich text present (cRun u16, 4 bytes per run)
//!
//! The hard part: a string's *character data* can be split across a CONTINUE
//! record boundary, and at each boundary a **fresh grbit byte** is inserted
//! that re-declares compressed vs UTF-16 for the *remaining* characters. The
//! rich-run array and phonetic blob can also straddle boundaries (but never
//! restate a grbit — only the character payload does).

use super::biff;

/// A cursor over the merged SST byte stream that knows where the CONTINUE
/// boundaries are, so it can consume a fresh grbit byte whenever character
/// data resumes after a boundary.
struct SstCursor<'a> {
    data: &'a [u8],
    /// Sorted byte offsets at which a CONTINUE block begins.
    breaks: &'a [usize],
    pos: usize,
}

impl<'a> SstCursor<'a> {
    fn new(data: &'a [u8], breaks: &'a [usize]) -> Self {
        SstCursor {
            data,
            breaks,
            pos: 0,
        }
    }

    /// Is `pos` exactly at (or past) the next continue boundary?
    fn at_break(&self) -> bool {
        self.breaks.contains(&self.pos)
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn u8(&mut self) -> u8 {
        let v = self.data.get(self.pos).copied().unwrap_or(0);
        self.pos += 1;
        v
    }

    fn u16(&mut self) -> u16 {
        let v = biff::u16le(self.data, self.pos);
        self.pos += 2;
        v
    }

    fn u32(&mut self) -> u32 {
        let v = biff::u32le(self.data, self.pos);
        self.pos += 4;
        v
    }

    fn skip(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.data.len());
    }

    /// Read `cch` characters starting in the current grbit mode `compressed`,
    /// crossing CONTINUE boundaries (each of which prepends a fresh grbit byte
    /// that may flip the compression mode for the remaining characters).
    fn read_chars(&mut self, cch: usize, mut compressed: bool) -> String {
        let mut units: Vec<u16> = Vec::with_capacity(cch);
        let mut read = 0;
        while read < cch {
            if self.pos >= self.data.len() {
                break;
            }
            // If we're exactly at a continuation boundary, consume the fresh
            // grbit byte that restates the compression for remaining chars.
            if self.at_break() {
                let grbit = self.u8();
                compressed = grbit & 0x01 == 0;
            }
            if compressed {
                units.push(self.u8() as u16);
            } else {
                if self.remaining() < 2 {
                    // A UTF-16 unit split across the boundary is malformed for
                    // our purposes; bail out gracefully.
                    break;
                }
                units.push(self.u16());
            }
            read += 1;
        }
        String::from_utf16_lossy(&units)
    }
}

/// Parse the SST record payload into the table of unique strings.
///
/// `data` is the merged SST+CONTINUE payload; `breaks` are the byte offsets
/// where each CONTINUE block started (from [`biff::RawRecord::continue_breaks`]).
pub fn parse_sst(data: &[u8], breaks: &[usize]) -> Vec<String> {
    let mut out = Vec::new();
    if data.len() < 8 {
        return out;
    }
    let mut cur = SstCursor::new(data, breaks);
    let _total = cur.u32();
    let unique = cur.u32();

    for _ in 0..unique {
        if cur.pos >= data.len() {
            break;
        }
        // A string header (cch + grbit) is never split across a boundary in
        // practice; Excel keeps at least these 3 bytes together. But the *char
        // data* may be.
        let cch = cur.u16() as usize;
        let grbit = cur.u8();
        let compressed = grbit & 0x01 == 0;
        let rich = grbit & 0x08 != 0;
        let ext = grbit & 0x04 != 0;

        let c_run = if rich { cur.u16() as usize } else { 0 };
        let cch_ext = if ext { cur.u32() as usize } else { 0 };

        let s = cur.read_chars(cch, compressed);
        out.push(s);

        // Skip the rich-text run formatting (4 bytes each) and the phonetic
        // extended blob; these may also cross boundaries but carry no fresh
        // grbit, so a flat skip is correct.
        if rich {
            cur.skip(c_run * 4);
        }
        if ext {
            cur.skip(cch_ext);
        }
    }

    out
}

/// Build SST record bytes (the record body, *without* the 4-byte BIFF header),
/// splitting into the SST record plus CONTINUE records as needed. Returns a
/// fully-framed byte stream: `[SST hdr][body...][CONTINUE hdr][more]...`.
pub fn build_sst_records(strings: &[String], total_refs: u32) -> Vec<u8> {
    // First serialize the logical payload: 8-byte header then each string.
    // Then we re-frame it into <=8224-byte record chunks, inserting a fresh
    // grbit byte at each boundary that lands inside a string's char data.
    //
    // To keep the grbit rule simple and always-correct, we serialize strings
    // and track, for every byte, whether it is part of a string's character
    // payload and what its compression flag is — so when a record fills up
    // mid-string we can resume with the right fresh grbit byte.

    // Build a flat list of "tokens": header bytes and per-string segments.
    let mut framer = SstFramer::new();
    framer.push_bytes(&total_refs.to_le_bytes());
    framer.push_bytes(&(strings.len() as u32).to_le_bytes());

    for s in strings {
        let chars: Vec<u16> = s.encode_utf16().collect();
        let compressed = chars.iter().all(|&c| c <= 0xFF);
        // String header: cch (u16) + grbit (u8). Keep these together.
        let grbit: u8 = if compressed { 0x00 } else { 0x01 };
        framer.push_header(&(chars.len() as u16).to_le_bytes(), grbit);
        // Character payload, which may need a fresh grbit on continuation.
        framer.push_chars(&chars, compressed);
    }

    framer.finish()
}

/// Frames SST logical bytes into BIFF records, honoring the 8224-byte limit and
/// inserting a fresh grbit byte whenever a string's character payload resumes in
/// a new CONTINUE record.
struct SstFramer {
    records: Vec<Vec<u8>>,
    cur: Vec<u8>,
}

impl SstFramer {
    fn new() -> Self {
        SstFramer {
            records: Vec::new(),
            cur: Vec::new(),
        }
    }

    fn room(&self) -> usize {
        biff::MAX_RECORD_DATA - self.cur.len()
    }

    fn flush(&mut self) {
        if !self.cur.is_empty() {
            self.records.push(std::mem::take(&mut self.cur));
        }
    }

    /// Push raw bytes that must not be split in a way that needs a grbit (e.g.
    /// the SST 8-byte header). If they don't fit, start a new record.
    fn push_bytes(&mut self, bytes: &[u8]) {
        if bytes.len() > self.room() {
            self.flush();
        }
        self.cur.extend_from_slice(bytes);
    }

    /// Push a string header (cch + grbit) atomically — never split.
    fn push_header(&mut self, cch: &[u8; 2], grbit: u8) {
        if self.room() < 3 {
            self.flush();
        }
        self.cur.extend_from_slice(cch);
        self.cur.push(grbit);
    }

    /// Push character payload, splitting across records. When a split occurs
    /// mid-string, the new record begins with a fresh grbit byte.
    fn push_chars(&mut self, chars: &[u16], compressed: bool) {
        let unit = if compressed { 1 } else { 2 };
        let mut i = 0;
        while i < chars.len() {
            if self.room() < unit {
                // Start a new record; it must lead with a fresh grbit byte.
                self.flush();
                self.cur.push(if compressed { 0x00 } else { 0x01 });
            }
            // How many units fit in the remaining room?
            let fit = self.room() / unit;
            let take = fit.min(chars.len() - i);
            for &c in &chars[i..i + take] {
                if compressed {
                    self.cur.push(c as u8);
                } else {
                    self.cur.extend_from_slice(&c.to_le_bytes());
                }
            }
            i += take;
        }
    }

    /// Emit the framed byte stream with proper SST/CONTINUE record headers.
    fn finish(mut self) -> Vec<u8> {
        self.flush();
        let mut out = Vec::new();
        for (idx, body) in self.records.iter().enumerate() {
            let typ = if idx == 0 { biff::SST } else { biff::CONTINUE };
            out.extend_from_slice(&typ.to_le_bytes());
            out.extend_from_slice(&(body.len() as u16).to_le_bytes());
            out.extend_from_slice(body);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an SST payload by hand with a forced CONTINUE boundary mid-string,
    /// then parse it back.
    #[test]
    fn sst_continue_boundary() {
        // Two strings: a long compressed one we split, and a short one.
        let s1: String = "A".repeat(20);
        let s2 = "hi".to_string();

        // Manually build a merged buffer and breaks list emulating what the
        // record iterator produces.
        // Header: total=2, unique=2.
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        // String 1 header: cch=20, grbit=0 (compressed).
        data.extend_from_slice(&20u16.to_le_bytes());
        data.push(0x00);
        // First 12 chars of s1 in this "record".
        for b in s1.bytes().take(12) {
            data.push(b);
        }
        // CONTINUE boundary HERE.
        let brk = data.len();
        // Fresh grbit byte (still compressed) then remaining 8 chars.
        data.push(0x00);
        for b in s1.bytes().skip(12) {
            data.push(b);
        }
        // String 2 header + data, no boundary.
        data.extend_from_slice(&2u16.to_le_bytes());
        data.push(0x00);
        data.extend_from_slice(s2.as_bytes());

        let parsed = parse_sst(&data, &[brk]);
        assert_eq!(parsed, vec![s1, s2]);
    }

    #[test]
    fn sst_continue_flips_to_utf16() {
        // A compressed string that switches grbit to UTF-16 at the boundary.
        // cch=4, first 2 chars compressed ('a','b'), then boundary flips to
        // UTF-16 for the last 2 chars ('c','d').
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes()); // total
        data.extend_from_slice(&1u32.to_le_bytes()); // unique
        data.extend_from_slice(&4u16.to_le_bytes()); // cch
        data.push(0x00); // grbit compressed
        data.push(b'a');
        data.push(b'b');
        let brk = data.len();
        data.push(0x01); // fresh grbit -> UTF-16
        data.extend_from_slice(&(b'c' as u16).to_le_bytes());
        data.extend_from_slice(&(b'd' as u16).to_le_bytes());

        let parsed = parse_sst(&data, &[brk]);
        assert_eq!(parsed, vec!["abcd".to_string()]);
    }

    #[test]
    fn sst_build_and_reparse_roundtrip() {
        let strings: Vec<String> = vec![
            "hello".into(),
            "world".into(),
            "café".into(), // forces UTF-16
            "x".repeat(5000),
            "y".repeat(5000), // together they exceed one record
        ];
        let framed = build_sst_records(&strings, strings.len() as u32);

        // Re-parse using the record iterator to merge CONTINUE blocks.
        let mut recs = biff::Records::new(&framed);
        let sst = recs.next().unwrap();
        assert_eq!(sst.typ, biff::SST);
        let parsed = parse_sst(&sst.data, &sst.continue_breaks);
        assert_eq!(parsed, strings);
    }
}
