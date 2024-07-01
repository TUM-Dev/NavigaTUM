use core::fmt::Formatter;
use std::fmt;

enum OrMore<T> {
    Value(T),
    More,
}

impl<T: fmt::Debug> fmt::Debug for OrMore<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            OrMore::Value(t) => fmt::Debug::fmt(t, f),
            OrMore::More => write!(f, "..."),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LimitedVec<T>(pub Vec<T>);

impl<T> LimitedVec<T> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
