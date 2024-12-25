use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::limited::OrMore;

#[derive(Serialize, Deserialize, Clone, Default, utoipa::ToSchema)]
pub struct LimitedHashMap<K: Eq + Hash, V>(pub HashMap<K, V>);

impl<K: Eq + Hash, V> From<HashMap<K, V>> for LimitedHashMap<K, V> {
    fn from(value: HashMap<K, V>) -> Self {
        LimitedHashMap(value)
    }
}

const LIMIT: usize = 3;
impl<K: fmt::Debug + Eq + Hash + Clone + Ord, V: fmt::Debug + Clone> fmt::Debug
    for LimitedHashMap<K, V>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut collection = self.0.clone().into_iter().collect::<Vec<(K, V)>>();
        collection.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));
        if self.0.len() <= LIMIT {
            f.debug_map().entries(collection).finish()
        } else {
            f.debug_map()
                .entries(
                    collection
                        .into_iter()
                        .take(LIMIT)
                        .map(|(k, v)| (OrMore::Value(k), OrMore::Value(v)))
                        .chain([(OrMore::More, OrMore::More)]),
                )
                .finish()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_limited_output() {
        let w: LimitedHashMap<u32, u32> = LimitedHashMap(HashMap::new());
        assert_eq!(format!("{w:?}"), "{}");
        let w = LimitedHashMap(HashMap::from([(1, 1)]));
        assert_eq!(format!("{w:?}"), "{1: 1}");
        let w = LimitedHashMap(HashMap::from([(1, 1), (2, 2)]));
        assert_eq!(format!("{w:?}"), "{1: 1, 2: 2}");
        let w = LimitedHashMap(HashMap::from([(1, 1), (2, 2), (3, 3)]));
        assert_eq!(format!("{w:?}"), "{1: 1, 2: 2, 3: 3}");
        let w = LimitedHashMap(HashMap::from([(1, 1), (2, 2), (3, 3), (4, 4)]));
        assert_eq!(format!("{w:?}"), "{1: 1, 2: 2, 3: 3, ...: ...}");
        let w = LimitedHashMap(HashMap::from([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]));
        assert_eq!(format!("{w:?}"), "{1: 1, 2: 2, 3: 3, ...: ...}");
    }
}
