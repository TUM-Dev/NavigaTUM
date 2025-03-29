use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EncodedPolyline {
    /// The number of points in the string
    pub length: i64,
    /// The encoded points of the polyline using the Google polyline encoding with precision 7.
    pub points: String,
}

impl EncodedPolyline {
    /// The number of points in the string
    #[inline]
    pub fn length(mut self, value: i64) -> Self {
        self.length = value;
        self
    }

    /// The encoded points of the polyline using the Google polyline encoding with precision 7.
    #[inline]
    pub fn points(mut self, value: impl ToString) -> Self {
        self.points = value.to_string();
        self
    }
}
