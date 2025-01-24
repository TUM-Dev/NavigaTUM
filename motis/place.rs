use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Place {
    /// arrival time
    pub arrival: Option<String>,
    /// departure time
    pub departure: Option<String>,
    /// latitude
    pub lat: f64,
    /// level according to OpenStreetMap
    pub level: f64,
    /// longitude
    pub lon: f64,
    /// name of the transit stop / PoI / address
    pub name: String,
    /// scheduled arrival time
    #[serde(rename = "scheduledArrival")]
    pub scheduled_arrival: Option<String>,
    /// scheduled departure time
    #[serde(rename = "scheduledDeparture")]
    pub scheduled_departure: Option<String>,
    /// scheduled track from the static schedule timetable dataset
    #[serde(rename = "scheduledTrack")]
    pub scheduled_track: Option<String>,
    /// The ID of the stop. This is often something that users don't care about.
    #[serde(rename = "stopId")]
    pub stop_id: Option<String>,
    /// The current track/platform information, updated with real-time updates if available.
    /// Can be missing if neither real-time updates nor the schedule timetable contains track information.
    pub track: Option<String>,
    #[serde(rename = "vertexType")]
    pub vertex_type: Option<crate::vertex_type::VertexType>,
}

impl Place {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> PlaceBuilder<
        crate::generics::MissingLat,
        crate::generics::MissingLevel,
        crate::generics::MissingLon,
        crate::generics::MissingName,
    > {
        PlaceBuilder {
            body: Default::default(),
            _lat: core::marker::PhantomData,
            _level: core::marker::PhantomData,
            _lon: core::marker::PhantomData,
            _name: core::marker::PhantomData,
        }
    }
}

impl Into<Place>
    for PlaceBuilder<
        crate::generics::LatExists,
        crate::generics::LevelExists,
        crate::generics::LonExists,
        crate::generics::NameExists,
    >
{
    fn into(self) -> Place {
        self.body
    }
}

/// Builder for [`Place`](./struct.Place.html) object.
#[derive(Debug, Clone)]
pub struct PlaceBuilder<Lat, Level, Lon, Name> {
    body: self::Place,
    _lat: core::marker::PhantomData<Lat>,
    _level: core::marker::PhantomData<Level>,
    _lon: core::marker::PhantomData<Lon>,
    _name: core::marker::PhantomData<Name>,
}

impl<Lat, Level, Lon, Name> PlaceBuilder<Lat, Level, Lon, Name> {
    /// arrival time
    #[inline]
    pub fn arrival(mut self, value: impl Into<String>) -> Self {
        self.body.arrival = Some(value.into());
        self
    }

    /// departure time
    #[inline]
    pub fn departure(mut self, value: impl Into<String>) -> Self {
        self.body.departure = Some(value.into());
        self
    }

    /// latitude
    #[inline]
    pub fn lat(
        mut self,
        value: impl Into<f64>,
    ) -> PlaceBuilder<crate::generics::LatExists, Level, Lon, Name> {
        self.body.lat = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// level according to OpenStreetMap
    #[inline]
    pub fn level(
        mut self,
        value: impl Into<f64>,
    ) -> PlaceBuilder<Lat, crate::generics::LevelExists, Lon, Name> {
        self.body.level = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// longitude
    #[inline]
    pub fn lon(
        mut self,
        value: impl Into<f64>,
    ) -> PlaceBuilder<Lat, Level, crate::generics::LonExists, Name> {
        self.body.lon = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// name of the transit stop / PoI / address
    #[inline]
    pub fn name(
        mut self,
        value: impl Into<String>,
    ) -> PlaceBuilder<Lat, Level, Lon, crate::generics::NameExists> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// scheduled arrival time
    #[inline]
    pub fn scheduled_arrival(mut self, value: impl Into<String>) -> Self {
        self.body.scheduled_arrival = Some(value.into());
        self
    }

    /// scheduled departure time
    #[inline]
    pub fn scheduled_departure(mut self, value: impl Into<String>) -> Self {
        self.body.scheduled_departure = Some(value.into());
        self
    }

    /// scheduled track from the static schedule timetable dataset
    #[inline]
    pub fn scheduled_track(mut self, value: impl Into<String>) -> Self {
        self.body.scheduled_track = Some(value.into());
        self
    }

    /// The ID of the stop. This is often something that users don't care about.
    #[inline]
    pub fn stop_id(mut self, value: impl Into<String>) -> Self {
        self.body.stop_id = Some(value.into());
        self
    }

    /// The current track/platform information, updated with real-time updates if available.
    /// Can be missing if neither real-time updates nor the schedule timetable contains track information.
    ///
    #[inline]
    pub fn track(mut self, value: impl Into<String>) -> Self {
        self.body.track = Some(value.into());
        self
    }

    #[inline]
    pub fn vertex_type(mut self, value: crate::vertex_type::VertexType) -> Self {
        self.body.vertex_type = Some(value.into());
        self
    }
}
