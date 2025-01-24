use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EncodedPolyline {
    /// The number of points in the string
    pub length: i64,
    /// The encoded points of the polyline using the Google polyline encoding with precision 7.
    pub points: String,
}

impl EncodedPolyline {
    /// Create a builder for this object.
    #[inline]
    pub fn builder(
    ) -> EncodedPolylineBuilder<crate::generics::MissingLength, crate::generics::MissingPoints>
    {
        EncodedPolylineBuilder {
            body: Default::default(),
            _length: core::marker::PhantomData,
            _points: core::marker::PhantomData,
        }
    }
}

impl Into<EncodedPolyline>
    for EncodedPolylineBuilder<crate::generics::LengthExists, crate::generics::PointsExists>
{
    fn into(self) -> EncodedPolyline {
        self.body
    }
}

/// Builder for [`EncodedPolyline`](./struct.EncodedPolyline.html) object.
#[derive(Debug, Clone)]
pub struct EncodedPolylineBuilder<Length, Points> {
    body: self::EncodedPolyline,
    _length: core::marker::PhantomData<Length>,
    _points: core::marker::PhantomData<Points>,
}

impl<Length, Points> EncodedPolylineBuilder<Length, Points> {
    /// The number of points in the string
    #[inline]
    pub fn length(
        mut self,
        value: impl Into<i64>,
    ) -> EncodedPolylineBuilder<crate::generics::LengthExists, Points> {
        self.body.length = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// The encoded points of the polyline using the Google polyline encoding with precision 7.
    #[inline]
    pub fn points(
        mut self,
        value: impl Into<String>,
    ) -> EncodedPolylineBuilder<Length, crate::generics::PointsExists> {
        self.body.points = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
