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

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits.iter().enumerate().flat_map(|(word_idx, &word)| {
            (0..64).filter_map(move |bit_idx| {
                if (word & (1u64 << bit_idx)) != 0 {
                    Some(word_idx * 64 + bit_idx)
                } else {
                    None
                }
            })
        })
    }

    pub fn as_raw_u64(&self) -> u64 {
        self.bits.first().copied().unwrap_or(0)
    }

    pub fn as_raw_bits(&self) -> &[u64] {
        &self.bits
    }
}

impl std::fmt::Debug for AffinityMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cores: Vec<usize> = self.iter().collect();

        f.debug_struct("AffinityMask")
            .field("cores", &cores)
            .field("count", &cores.len())
            .finish()
    }
}

impl std::fmt::Display for AffinityMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cores: Vec<usize> = self.iter().collect();

        if cores.is_empty() {
            write!(f, "AffinityMask(empty)")
        } else {
            write!(f, "AffinityMask({:?})", cores)
        }
    }
}

impl FromIterator<usize> for AffinityMask {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut mask = AffinityMask::empty();

        for id in iter {
            mask.add(id);
        }

        mask
    }
}

impl IntoIterator for &AffinityMask {
    type Item = usize;
    type IntoIter = std::vec::IntoIter<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}
