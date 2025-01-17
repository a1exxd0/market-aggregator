use std::{collections::VecDeque, fmt::Debug};

/// Queriable implementation of an in-memory structure
/// to hold ordered data in a map-like K/V fashion.
///
/// Capacity is bounded below by `1` and defaulted to 100K.
pub struct TimeSeriesArray<K, V>
where
    K: Copy + Ord + Debug,
    V: Clone + Debug,
{
    keys: VecDeque<K>,
    values: VecDeque<V>,
    max_elem_count: usize,
}

pub enum TimeSeriesError {
    KeyExists,
}

impl<K, V> TimeSeriesArray<K, V>
where
    K: Copy + Ord + Debug,
    V: Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            keys: VecDeque::new(),
            values: VecDeque::new(),
            max_elem_count: 100000,
        }
    }

    /// Create an array with a set capacity, bounded below by 1.
    pub fn new_with_capacity(max_elem_count: usize) -> Self {
        Self {
            keys: VecDeque::new(),
            values: VecDeque::new(),
            max_elem_count: std::cmp::max(1, max_elem_count),
        }
    }

    /// Insert a value into the structure
    pub fn insert(&mut self, key: K, val: &V) -> Result<(), TimeSeriesError> {
        if self.keys.len() >= self.max_elem_count {
            self.remove_element();
        }

        if let Some(most_recent) = self.keys.back() {
            if key < *most_recent {
                return self.insert_not_ordered(key, val);
            }
        }

        self.keys.push_back(key);
        self.values.push_back(val.clone());
        Ok(())
    }

    /// Returns the value corresponding to an exact key, or None.
    pub fn find_key(&self, key: &K) -> Option<&V> {
        match self.keys.binary_search(&key) {
            Ok(i) => Some(&self.values[i]),
            Err(_) => None,
        }
    }

    /// Returns the last value equal to or less than some key, or
    /// none if there is no such key present satisfying.
    pub fn last_value_for_key(&self, key: &K) -> Option<(K, &V)> {
        match self.get_index_last_value_for_key(&key) {
            None => None,
            Some(i) => Some((self.keys[i], &self.values[i])),
        }
    }

    /// Returns a splice to all indexes in the range [lower_bound, upper_bound).
    pub fn range_query(&self, lower_bound: K, upper_bound: K) -> impl Iterator<Item = (&K, &V)> {
        let idx_lb = self.keys.binary_search(&lower_bound).unwrap_or_else(|x| x);
        let idx_ub = std::cmp::min(
            self.values.len(),
            self.keys.binary_search(&upper_bound).unwrap_or_else(|x| x),
        );

        if idx_lb >= self.keys.len() || idx_lb >= idx_ub {
            return self.keys.range(0..0).zip(self.values.range(0..0));
        }

        self.keys
            .range(idx_lb..idx_ub)
            .zip(self.values.range(idx_lb..idx_ub))
    }

    #[inline]
    fn get_index_last_value_for_key(&self, key: &K) -> Option<usize> {
        match self.keys.binary_search(&key) {
            Ok(i) => Some(i),
            Err(i) => match i {
                0 => None,
                i => Some(i - 1),
            },
        }
    }

    #[inline]
    fn insert_not_ordered(&mut self, key: K, val: &V) -> Result<(), TimeSeriesError> {
        match self.keys.binary_search(&key) {
            Ok(_) => return Err(TimeSeriesError::KeyExists),
            Err(pos) => {
                self.keys.insert(pos, key);
                self.values.insert(pos, val.clone());
                Ok(())
            }
        }
    }

    #[inline]
    fn remove_element(&mut self) -> Option<(K, V)> {
        match self.keys.len() {
            0 => None,
            _ => Some((self.keys.pop_front()?, self.values.pop_front()?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::TimeSeriesArray;

    fn create_test_data() -> Vec<(std::time::Duration, u64)> {
        let mut data = Vec::new();

        for i in 0..1000 {
            data.push((Duration::from_secs(i), (i + 41) % 23));
        }

        data
    }

    fn create_test_data_late_range() -> Vec<(std::time::Duration, u64)> {
        let mut data = Vec::new();

        for i in 500..2000 {
            data.push((Duration::from_secs(i), (i + 41) % 23));
        }

        data
    }

    fn create_test_array(vec: &Vec<(std::time::Duration, u64)>) -> TimeSeriesArray<Duration, u64> {
        let mut arr = TimeSeriesArray::new();
        for (time, val) in vec {
            match arr.insert(time.clone(), val) {
                Ok(_) => (),
                Err(_) => panic!("didnt expect test insertion failure"),
            }
        }

        arr
    }

    #[test]
    fn test_data_insertions() {
        let created_data = create_test_data();

        let mut arr = TimeSeriesArray::new();
        for (time, val) in created_data {
            match arr.insert(time, &val) {
                Ok(_) => (),
                Err(_) => panic!("didnt expect test insertion failure"),
            }
        }
    }

    #[test]
    fn verify_item_in_time_exists() {
        let arr = create_test_array(&create_test_data());
        assert!(arr.find_key(&Duration::from_secs(943)).is_some())
    }

    #[test]
    fn verify_correct_last_time_when_exists() {
        let arr = create_test_array(&create_test_data());
        let first_entry = arr.last_value_for_key(&Duration::from_millis(500));
        assert!(first_entry.is_some());
        if let Some((time, _)) = first_entry {
            assert_eq!(time, Duration::from_millis(0));
        }
        assert!(arr.last_value_for_key(&Duration::from_millis(0)).is_some());
    }

    #[test]
    fn verify_none_return_when_none_present() {
        let empty_arr = TimeSeriesArray::<Duration, u64>::new_with_capacity(100);
        assert_eq!(empty_arr.find_key(&Duration::from_millis(0)), None);

        let arr = create_test_array(&create_test_data_late_range());
        assert_eq!(arr.find_key(&Duration::from_secs(1)), None);
    }

    #[test]
    fn test_working_range_query() {
        let mut arr = TimeSeriesArray::new_with_capacity(50);
        for i in 0..100 {
            let _ = arr.insert(Duration::from_secs(i), &(i + 1));
        }

        // expect [50-99]
        let iter = arr.range_query(Duration::from_secs(49), Duration::from_secs(1000));
        let mut expected = 50;

        for (time, val) in iter {
            assert_eq!(*time, Duration::from_secs(expected));
            assert_eq!(*val, expected + 1);
            expected += 1;
        }
    }
}
