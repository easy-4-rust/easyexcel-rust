/// BIFF8 `CryptoAPI` 使用的 RC4 状态机。
///
/// 对应 Java：`javax.crypto.Cipher`（`RC4`）及 POI `CryptoAPICipherOutputStream`。
pub(super) struct Biff8Rc4State {
    state: [u8; 256],
    i: u8,
    j: u8,
}

impl Biff8Rc4State {
    pub(super) fn new(key: &[u8]) -> Self {
        debug_assert!(!key.is_empty());
        let mut state = [0u8; 256];
        for (index, value) in state.iter_mut().enumerate() {
            *value = index as u8;
        }
        let mut j = 0u8;
        for index in 0..256usize {
            j = j
                .wrapping_add(state[index])
                .wrapping_add(key[index % key.len()]);
            state.swap(index, usize::from(j));
        }
        Self { state, i: 0, j: 0 }
    }

    pub(super) fn next_byte(&mut self) -> u8 {
        self.i = self.i.wrapping_add(1);
        self.j = self.j.wrapping_add(self.state[usize::from(self.i)]);
        self.state.swap(usize::from(self.i), usize::from(self.j));
        let index = self.state[usize::from(self.i)].wrapping_add(self.state[usize::from(self.j)]);
        self.state[usize::from(index)]
    }

    pub(super) fn apply(&mut self, bytes: &mut [u8]) {
        for byte in bytes {
            *byte ^= self.next_byte();
        }
    }
}
