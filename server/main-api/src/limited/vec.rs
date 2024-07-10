use std::fmt;
use std::vec::IntoIter;

use serde::{Deserialize, Serialize};

use crate::limited::OrMore;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LimitedVec<T>(pub Vec<T>);

impl<T> AsRef<[T]> for LimitedVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T> IntoIterator for LimitedVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> LimitedVec<T> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<T> From<Vec<T>> for LimitedVec<T> {
    fn from(value: Vec<T>) -> Self {
        LimitedVec(value)
    }
}

const LIMIT: usize = 3;
impl<T: fmt::Debug> fmt::Debug for LimitedVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() <= LIMIT {
            f.debug_list().entries(self.0.iter().take(LIMIT)).finish()
        } else {
            f.debug_list()
                .entries(
                    self.0
                        .iter()
                        .take(LIMIT)
                        .map(OrMore::Value)
                        .chain([OrMore::More]),
                )
                .finish()
        }
    }
}
impl<T> FromIterator<T> for LimitedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut c = Vec::new();

        for i in iter {
            c.push(i);
        }

        LimitedVec(c)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_limited_output() {
        let w: LimitedVec<u32> = LimitedVec(vec![]);
        assert_eq!(format!("{w:?}"), "[]");
        let w = LimitedVec(vec![1]);
        assert_eq!(format!("{w:?}"), "[1]");
        let w = LimitedVec(vec![1, 2]);
        assert_eq!(format!("{w:?}"), "[1, 2]");
        let w = LimitedVec(vec![1, 2, 3]);
        assert_eq!(format!("{w:?}"), "[1, 2, 3]");
        let w = LimitedVec(vec![1, 2, 3, 4]);
        assert_eq!(format!("{w:?}"), "[1, 2, 3, ...]");
        let w = LimitedVec(vec![1, 2, 3, 4, 5]);
        assert_eq!(format!("{w:?}"), "[1, 2, 3, ...]");
    }
}
