use serde::{Deserialize, Serialize};

/// Object containing duration if a path was found or none if no path was found
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Duration {
    /// duration in seconds if a path was found, otherwise missing
    pub duration: Option<f64>,
}

impl Duration {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> Self {
        Duration::default()
    }
    /// duration in seconds if a path was found, otherwise missing
    #[inline]
    pub fn duration(mut self, value: f64) -> Self {
        self.duration = Some(value);
        self
    }
}
