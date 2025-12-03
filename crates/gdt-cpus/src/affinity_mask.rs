//! CPU affinity mask for specifying sets of logical processors.
//!
//! This module provides the [`AffinityMask`] type, which represents a set of
//! logical processor IDs that a thread may run on. Unlike pinning to a single
//! core, affinity masks allow threads to migrate between a specified set of
//! cores, reducing scheduling latency while still constraining execution.

/// A cross-platform CPU affinity mask representing a set of logical processors.
///
/// An `AffinityMask` specifies which logical processors (CPU threads) a thread
/// is allowed to execute on. This is useful for:
///
/// - Restricting latency-sensitive threads to performance cores (P-cores)
/// - Restricting background threads to efficiency cores (E-cores)
/// - Allowing thread migration within a subset of cores to reduce scheduling latency
///
/// # Example
///
/// ```
/// use gdt_cpus::AffinityMask;
///
/// // Create a mask for cores 0, 1, and 2
/// let mask = AffinityMask::from_cores(&[0, 1, 2]);
/// assert_eq!(mask.count(), 3);
/// assert!(mask.contains(0));
/// assert!(mask.contains(1));
/// assert!(mask.contains(2));
/// assert!(!mask.contains(3));
/// ```
///
/// # Platform Behavior
///
/// When used with [`set_thread_affinity`](crate::set_thread_affinity):
///
/// - **Linux/Windows**: The mask is applied directly to constrain thread execution
/// - **macOS**: Returns `Error::Unsupported`; use QoS classes instead
#[derive(Clone, PartialEq, Eq, Default)]
pub struct AffinityMask {
    /// Bitset stored as multiple u64 words to support >64 cores.
    /// bits[0] contains cores 0-63, bits[1] contains cores 64-127, etc.
    bits: Vec<u64>,
}

impl AffinityMask {
    /// Creates an empty affinity mask with no cores set.
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let mask = AffinityMask::empty();
    /// assert!(mask.is_empty());
    /// assert_eq!(mask.count(), 0);
    /// ```
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates an affinity mask with a single core set.
    ///
    /// # Arguments
    ///
    /// * `logical_core_id` - The logical processor ID to include in the mask
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let mask = AffinityMask::single(5);
    /// assert_eq!(mask.count(), 1);
    /// assert!(mask.contains(5));
    /// ```
    pub fn single(logical_core_id: usize) -> Self {
        let mut mask = Self::empty();

        mask.add(logical_core_id);

        mask
    }

    /// Creates an affinity mask from a slice of core IDs.
    ///
    /// # Arguments
    ///
    /// * `core_ids` - Slice of logical processor IDs to include
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let mask = AffinityMask::from_cores(&[0, 2, 4, 6]);
    /// assert_eq!(mask.count(), 4);
    /// assert!(mask.contains(0));
    /// assert!(!mask.contains(1));
    /// assert!(mask.contains(2));
    /// ```
    pub fn from_cores(core_ids: &[usize]) -> Self {
        let mut mask = Self::empty();

        for &id in core_ids {
            mask.add(id);
        }

        mask
    }

    /// Adds a logical core to the mask.
    ///
    /// If the core is already in the mask, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `logical_core_id` - The logical processor ID to add
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let mut mask = AffinityMask::empty();
    /// mask.add(0);
    /// mask.add(1);
    /// assert_eq!(mask.count(), 2);
    /// ```
    pub fn add(&mut self, logical_core_id: usize) {
        let word_idx = logical_core_id / 64;
        let bit_idx = logical_core_id % 64;

        if word_idx >= self.bits.len() {
            self.bits.resize(word_idx + 1, 0);
        }

        self.bits[word_idx] |= 1u64 << bit_idx;
    }

    /// Removes a logical core from the mask.
    ///
    /// If the core is not in the mask, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `logical_core_id` - The logical processor ID to remove
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let mut mask = AffinityMask::from_cores(&[0, 1, 2]);
    /// mask.remove(1);
    /// assert_eq!(mask.count(), 2);
    /// assert!(!mask.contains(1));
    /// ```
    pub fn remove(&mut self, logical_core_id: usize) {
        let word_idx = logical_core_id / 64;
        let bit_idx = logical_core_id % 64;

        if word_idx < self.bits.len() {
            self.bits[word_idx] &= !(1u64 << bit_idx);
        }
    }

    /// Checks if a logical core is in the mask.
    ///
    /// # Arguments
    ///
    /// * `logical_core_id` - The logical processor ID to check
    ///
    /// # Returns
    ///
    /// `true` if the core is in the mask, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let mask = AffinityMask::from_cores(&[0, 2, 4]);
    /// assert!(mask.contains(0));
    /// assert!(!mask.contains(1));
    /// assert!(mask.contains(2));
    /// ```
    pub fn contains(&self, logical_core_id: usize) -> bool {
        let word_idx = logical_core_id / 64;
        let bit_idx = logical_core_id % 64;

        if word_idx >= self.bits.len() {
            return false;
        }

        (self.bits[word_idx] & (1u64 << bit_idx)) != 0
    }

    /// Returns the number of cores in the mask.
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let mask = AffinityMask::from_cores(&[0, 1, 2, 3]);
    /// assert_eq!(mask.count(), 4);
    /// ```
    pub fn count(&self) -> usize {
        self.bits.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// Returns `true` if the mask is empty (no cores set).
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let empty = AffinityMask::empty();
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = AffinityMask::single(0);
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|&w| w == 0)
    }

    /// Returns an iterator over the core IDs in the mask.
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let mask = AffinityMask::from_cores(&[1, 3, 5]);
    /// let cores: Vec<usize> = mask.iter().collect();
    /// assert_eq!(cores, vec![1, 3, 5]);
    /// ```
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

    /// Returns the union of this mask with another.
    ///
    /// The resulting mask contains all cores that are in either mask.
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let a = AffinityMask::from_cores(&[0, 1]);
    /// let b = AffinityMask::from_cores(&[1, 2]);
    /// let union = a.union(&b);
    /// assert_eq!(union.count(), 3);
    /// assert!(union.contains(0));
    /// assert!(union.contains(1));
    /// assert!(union.contains(2));
    /// ```
    pub fn union(&self, other: &AffinityMask) -> AffinityMask {
        let max_len = self.bits.len().max(other.bits.len());
        let mut bits = vec![0u64; max_len];

        for (i, &word) in self.bits.iter().enumerate() {
            bits[i] |= word;
        }
        for (i, &word) in other.bits.iter().enumerate() {
            bits[i] |= word;
        }

        AffinityMask { bits }
    }

    /// Returns the intersection of this mask with another.
    ///
    /// The resulting mask contains only cores that are in both masks.
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let a = AffinityMask::from_cores(&[0, 1, 2]);
    /// let b = AffinityMask::from_cores(&[1, 2, 3]);
    /// let intersection = a.intersection(&b);
    /// assert_eq!(intersection.count(), 2);
    /// assert!(intersection.contains(1));
    /// assert!(intersection.contains(2));
    /// ```
    pub fn intersection(&self, other: &AffinityMask) -> AffinityMask {
        let min_len = self.bits.len().min(other.bits.len());
        let bits: Vec<u64> = self.bits[..min_len]
            .iter()
            .zip(other.bits[..min_len].iter())
            .map(|(&a, &b)| a & b)
            .collect();

        AffinityMask { bits }
    }

    /// Returns the first 64 cores as a raw `u64` bitmask.
    ///
    /// This is useful for platform APIs that only support 64 cores.
    /// Cores beyond index 63 are not included.
    ///
    /// # Example
    ///
    /// ```
    /// use gdt_cpus::AffinityMask;
    ///
    /// let mask = AffinityMask::from_cores(&[0, 1, 63]);
    /// let raw = mask.as_raw_u64();
    /// assert_eq!(raw, 0x8000_0000_0000_0003);
    /// ```
    pub fn as_raw_u64(&self) -> u64 {
        self.bits.first().copied().unwrap_or(0)
    }

    /// Returns the raw bits as a slice.
    ///
    /// Each element represents 64 cores: `bits[0]` = cores 0-63,
    /// `bits[1]` = cores 64-127, etc.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_mask() {
        let mask = AffinityMask::empty();
        assert!(mask.is_empty());
        assert_eq!(mask.count(), 0);
        assert!(!mask.contains(0));
    }

    #[test]
    fn test_single_core() {
        let mask = AffinityMask::single(5);
        assert!(!mask.is_empty());
        assert_eq!(mask.count(), 1);
        assert!(mask.contains(5));
        assert!(!mask.contains(4));
    }

    #[test]
    fn test_from_cores() {
        let mask = AffinityMask::from_cores(&[0, 2, 4, 6]);
        assert_eq!(mask.count(), 4);
        assert!(mask.contains(0));
        assert!(!mask.contains(1));
        assert!(mask.contains(2));
    }

    #[test]
    fn test_add_remove() {
        let mut mask = AffinityMask::empty();
        mask.add(0);
        mask.add(1);
        assert_eq!(mask.count(), 2);

        mask.remove(0);
        assert_eq!(mask.count(), 1);
        assert!(!mask.contains(0));
        assert!(mask.contains(1));
    }

    #[test]
    fn test_high_core_ids() {
        let mut mask = AffinityMask::empty();
        mask.add(0);
        mask.add(64);
        mask.add(128);

        assert_eq!(mask.count(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(64));
        assert!(mask.contains(128));
        assert!(!mask.contains(63));
        assert!(!mask.contains(65));
    }

    #[test]
    fn test_iter() {
        let mask = AffinityMask::from_cores(&[1, 3, 5, 64, 65]);
        let cores: Vec<usize> = mask.iter().collect();
        assert_eq!(cores, vec![1, 3, 5, 64, 65]);
    }

    #[test]
    fn test_as_raw_u64() {
        let mask = AffinityMask::from_cores(&[0, 1, 63]);
        assert_eq!(mask.as_raw_u64(), 0x8000_0000_0000_0003);
    }

    #[test]
    fn test_from_iterator() {
        let mask: AffinityMask = vec![0, 2, 4].into_iter().collect();
        assert_eq!(mask.count(), 3);
        assert!(mask.contains(0));
        assert!(mask.contains(2));
        assert!(mask.contains(4));
    }
}
