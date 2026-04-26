use std::fmt;
use std::fmt::Formatter;

pub mod hash_map;
pub mod vec;

enum OrMore<T> {
    Value(T),
    More,
}

impl<T: fmt::Debug> fmt::Debug for OrMore<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Value(t) => fmt::Debug::fmt(t, f),
            Self::More => write!(f, "..."),
        }
    }
}
