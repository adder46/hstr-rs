use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;

pub fn sort<T>(mut entries: Vec<T>) -> Vec<T>
where
    T: Clone + Eq + Hash
{
    let freq_map = frequency_map(&entries);
    let pos_map = position_map(&entries);
    entries.sort_unstable_by(|a, b| pos_map.get(b).unwrap().cmp(pos_map.get(a).unwrap()));
    entries.dedup();
    entries.sort_by(|a, b| freq_map.get(b).unwrap().cmp(freq_map.get(a).unwrap()));
    entries
}

fn frequency_map<T>(entries: &[T]) -> HashMap<T, usize>
where
    T: Clone + Eq + Hash
{
    let mut map = HashMap::new();
    for e in entries.iter() {
        *map.entry(e.clone()).or_insert(0) += 1;
    }
    map
}

fn position_map<T>(entries: &[T]) -> HashMap<T, usize>
where
    T: Clone + Eq + Hash
{
    let mut map = HashMap::new();
    for (pos, e) in entries.iter().enumerate() {
        map.insert(e.clone(), pos);
    }
    map
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sort() {
        let vec = vec![3, 2, 4, 6, 2, 4, 3, 3, 4, 5, 6, 3, 2, 4, 5, 5, 3];
        let vec = super::sort(vec);
        assert_eq!(vec, [3, 4, 5, 2, 6]);
    }
}