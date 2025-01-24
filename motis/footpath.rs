use serde::{Deserialize, Serialize};

/// footpath from one location to another
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Footpath {
    /// footpath duration in minutes according to GTFS (+heuristics)
    /// 
    /// [`None`] if the GTFS did not contain a footpath
    pub default: Option<f64>,
    /// footpath duration in minutes for the foot profile
    /// 
    /// [`None`] if no path was found with the foot profile
    pub foot: Option<f64>,
    pub to: crate::place::Place,
    /// footpath duration in minutes for the wheelchair profile
    /// 
    /// [`None`] if no path was found with the wheelchair profile
    pub wheelchair: Option<f64>,
    /// true if the wheelchair path uses an elevator
    /// 
    /// [`None`] if no path was found with the wheelchair profile
    pub wheelchair_uses_elevator: Option<bool>,
}

impl Footpath {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> FootpathBuilder<crate::generics::MissingTo> {
        FootpathBuilder {
            body: Default::default(),
            _to: core::marker::PhantomData,
        }
    }
}

impl Into<Footpath> for FootpathBuilder<crate::generics::ToExists> {
    fn into(self) -> Footpath {
        self.body
    }
}

/// Builder for [`Footpath`](./struct.Footpath.html) object.
#[derive(Debug, Clone)]
pub struct FootpathBuilder<To> {
    body: self::Footpath,
    _to: core::marker::PhantomData<To>,
}

impl<To> FootpathBuilder<To> {
    /// optional; missing if the GTFS did not contain a footpath
    /// footpath duration in minutes according to GTFS (+heuristics)
    ///
    #[inline]
    pub fn default(mut self, value: impl Into<f64>) -> Self {
        self.body.default = Some(value.into());
        self
    }

    /// optional; missing if no path was found with the foot profile
    /// footpath duration in minutes for the foot profile
    ///
    #[inline]
    pub fn foot(mut self, value: impl Into<f64>) -> Self {
        self.body.foot = Some(value.into());
        self
    }

    #[inline]
    pub fn to(
        mut self,
        value: crate::place::PlaceBuilder<
            crate::generics::LatExists,
            crate::generics::LevelExists,
            crate::generics::LonExists,
            crate::generics::NameExists,
        >,
    ) -> FootpathBuilder<crate::generics::ToExists> {
        self.body.to = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// optional; missing if no path was found with the wheelchair profile
    /// footpath duration in minutes for the wheelchair profile
    ///
    #[inline]
    pub fn wheelchair(mut self, value: impl Into<f64>) -> Self {
        self.body.wheelchair = Some(value.into());
        self
    }

    /// optional; missing if no path was found with the wheelchair profile
    /// true if the wheelchair path uses an elevator
    ///
    #[inline]
    pub fn wheelchair_uses_elevator(mut self, value: impl Into<bool>) -> Self {
        self.body.wheelchair_uses_elevator = Some(value.into());
        self
    }
}
