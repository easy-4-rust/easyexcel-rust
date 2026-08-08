/// BIFF8 SST/CONTINUE 分帧器。
///
/// 对应 Java：Apache POI `SSTSerializer` / `ContinuableRecordOutput`。字符区跨
/// CONTINUE 时写入新的压缩标志，格式 run 区跨记录时则不写该标志。
struct Biff8SstFramer {
    records: Vec<Vec<u8>>,
    current: Vec<u8>,
}

impl Biff8SstFramer {
    fn new(total_refs: u32, unique: u32) -> Self {
        let mut current = Vec::with_capacity(MAX_RECORD_DATA);
        current.extend_from_slice(&total_refs.to_le_bytes());
        current.extend_from_slice(&unique.to_le_bytes());
        Self {
            records: Vec::new(),
            current,
        }
    }

    fn room(&self) -> usize {
        MAX_RECORD_DATA.saturating_sub(self.current.len())
    }

    fn flush(&mut self) {
        if !self.current.is_empty() {
            self.records.push(std::mem::take(&mut self.current));
            self.current.reserve(MAX_RECORD_DATA);
        }
    }

    fn push_rich_text(&mut self, rich: &Biff8RichText) {
        let chars = rich.text.encode_utf16().collect::<Vec<_>>();
        let compressed = chars.iter().all(|unit| *unit <= 0xFF);
        let has_runs = !rich.runs.is_empty();
        let header_len = if has_runs { 5 } else { 3 };
        if self.room() < header_len {
            self.flush();
        }
        self.current.extend_from_slice(
            &u16::try_from(chars.len()).unwrap_or(u16::MAX).to_le_bytes(),
        );
        self.current
            .push(u8::from(!compressed) | (u8::from(has_runs) << 3));
        if has_runs {
            self.current.extend_from_slice(
                &u16::try_from(rich.runs.len())
                    .unwrap_or(u16::MAX)
                    .to_le_bytes(),
            );
        }
        self.push_chars(&chars, compressed);
        self.push_runs(&rich.runs);
    }

    fn push_chars(&mut self, chars: &[u16], compressed: bool) {
        let unit_width = if compressed { 1 } else { 2 };
        let mut offset = 0usize;
        while offset < chars.len() {
            if self.room() < unit_width {
                self.flush();
                // CONTINUE 恰好位于字符数据内部时必须先重申字符压缩模式。
                self.current.push(u8::from(!compressed));
            }
            let take = (self.room() / unit_width).min(chars.len() - offset);
            for unit in &chars[offset..offset + take] {
                if compressed {
                    self.current.push(u8::try_from(*unit).unwrap_or(b'?'));
                } else {
                    self.current.extend_from_slice(&unit.to_le_bytes());
                }
            }
            offset += take;
        }
    }

    fn push_runs(&mut self, runs: &[(u16, u16)]) {
        for (start, font) in runs {
            if self.room() < 4 {
                self.flush();
            }
            self.current.extend_from_slice(&start.to_le_bytes());
            self.current.extend_from_slice(&font.to_le_bytes());
        }
    }

    fn finish(mut self) -> Vec<u8> {
        self.flush();
        let mut out = Vec::new();
        for (index, payload) in self.records.iter().enumerate() {
            record(
                &mut out,
                if index == 0 { SST } else { CONTINUE },
                payload,
            );
        }
        out
    }
}

