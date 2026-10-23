use caches::{AdaptiveCache, Cache};
#[test]
fn promoting_ghost_cannot_evict_itself_during_replacement() {
    let mut cache = AdaptiveCache::<usize, ()>::new(4).unwrap();
    for key in [1, 9, 7, 11, 11, 10, 0, 9, 1, 0, 6] {
        cache.put(key, ());
    }
    cache.get(&10);
    for key in [5, 6, 7, 2, 9] {
        cache.put(key, ());
    }
    assert!(cache.get(&9).is_some());
}

#[test]
fn ghost_promotions_preserve_capacity_and_disjoint_lists() {
    for capacity in [4, 8, 16] {
        for seed in 1..=16u64 {
            let mut state = seed;
            let mut cache = AdaptiveCache::<usize, usize>::new(capacity).unwrap();
            for _ in 0..4096 {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let key = state as usize % (capacity * 3);
                match (state >> 16) % 5 {
                    0 => {
                        cache.remove(&key);
                    }
                    1 => {
                        if let Some(value) = cache.get(&key) {
                            assert_eq!(*value, key);
                        }
                    }
                    _ => {
                        cache.put(key, key);
                        assert_eq!(cache.peek(&key), Some(&key));
                    }
                }
                assert!(
                    cache.len() <= capacity,
                    "capacity={capacity} seed={seed} key={key} partition={} recent={} frequent={}",
                    cache.partition(),
                    cache.recent_len(),
                    cache.frequent_len()
                );
                let mut seen = std::collections::HashSet::new();
                for (key, value) in cache
                    .recent_iter()
                    .chain(cache.frequent_iter())
                    .chain(cache.recent_evict_iter())
                    .chain(cache.frequent_evict_iter())
                {
                    assert_eq!(key, value);
                    assert!(seen.insert(*key), "key appears in multiple ARC lists");
                }
            }
        }
    }
}
