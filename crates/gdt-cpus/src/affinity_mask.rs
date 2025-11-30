#[derive(Clone, PartialEq, Eq, Default)]
pub struct AffinityMask {
    bits: Vec<u64>,
}

impl AffinityMask {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn single(logical_core_id: usize) -> Self {
        let mut mask = Self::empty();

        mask.add(logical_core_id);

        mask
    }

    pub fn from_cores(core_ids: &[usize]) -> Self {
        let mut mask = Self::empty();

        for &id in core_ids {
            mask.add(id);
        }

        mask
    }

    pub fn add(&mut self, logical_core_id: usize) {
        let word_idx = logical_core_id / 64;
        let bit_idx = logical_core_id % 64;

        if word_idx >= self.bits.len() {
            self.bits.resize(word_idx + 1, 0);
        }

        self.bits[word_idx] |= 1u64 << bit_idx;
    }

    pub fn remove(&mut self, logical_core_id: usize) {
        let word_idx = logical_core_id / 64;
        let bit_idx = logical_core_id % 64;

        if word_idx < self.bits.len() {
            self.bits[word_idx] &= !(1u64 << bit_idx);
        }
    }

    pub fn contains(&self, logical_core_id: usize) -> bool {
        let word_idx = logical_core_id / 64;
        let bit_idx = logical_core_id % 64;

        if word_idx >= self.bits.len() {
            return false;
        }

        (self.bits[word_idx] & (1u64 << bit_idx)) != 0
    }

    pub fn count(&self) -> usize {
        self.bits.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|&w| w == 0)
    }
}
