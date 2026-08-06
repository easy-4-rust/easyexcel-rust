/// 对应 Java：无直接对应对象；Rust 架构扩展。 Iterator over BIFF records in a byte buffer. CONTINUE (0x003C) records are
/// automatically appended to the preceding record's data, and their boundary
/// offsets recorded in `continue_breaks`.
pub struct Records<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Records<'a> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn new(buf: &'a [u8]) -> Self {
        Records { buf, pos: 0 }
    }

    fn read_header(&self, at: usize) -> Option<(u16, usize)> {
        if at + 4 > self.buf.len() {
            return None;
        }
        let typ = u16::from_le_bytes([self.buf[at], self.buf[at + 1]]);
        let len = u16::from_le_bytes([self.buf[at + 2], self.buf[at + 3]]) as usize;
        Some((typ, len))
    }
}

impl Iterator for Records<'_> {
    type Item = RawRecord;

    fn next(&mut self) -> Option<RawRecord> {
        let (typ, len) = self.read_header(self.pos)?;
        let data_start = self.pos + 4;
        let data_end = (data_start + len).min(self.buf.len());
        let mut data = self.buf[data_start..data_end].to_vec();
        self.pos = data_end;

        // Merge any following CONTINUE records.
        let mut continue_breaks = Vec::new();
        while let Some((next_typ, next_len)) = self.read_header(self.pos) {
            if next_typ != CONTINUE {
                break;
            }
            let cstart = self.pos + 4;
            let cend = (cstart + next_len).min(self.buf.len());
            continue_breaks.push(data.len());
            data.extend_from_slice(&self.buf[cstart..cend]);
            self.pos = cend;
        }

        Some(RawRecord {
            typ,
            data,
            continue_breaks,
        })
    }
}

