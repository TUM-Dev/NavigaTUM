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
    pub fn builder() -> DurationBuilder {
        DurationBuilder {
            body: Default::default(),
        }
    }
}

impl Into<Duration> for DurationBuilder {
    fn into(self) -> Duration {
        self.body
    }
}

/// Builder for [`Duration`](./struct.Duration.html) object.
#[derive(Debug, Clone)]
pub struct DurationBuilder {
    body: self::Duration,
}

impl DurationBuilder {
    /// duration in seconds if a path was found, otherwise missing
    #[inline]
    pub fn duration(mut self, value: impl Into<f64>) -> Self {
        self.body.duration = Some(value.into());
        self
    }
}
