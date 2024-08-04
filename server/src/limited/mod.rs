use std::fmt;
use std::fmt::Formatter;

pub mod hash_map;
pub mod vec;

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
