use serde::{Deserialize, Serialize};

/// departure or arrival event at a stop
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopTime {
    pub agency_id: String,
    pub agency_name: String,
    pub agency_url: String,
    /// For transit legs, the headsign of the bus or train being used.
    /// For non-transit legs, null
    pub headsign: String,
    /// Transport mode for this leg
    pub mode: crate::mode::Mode,
    /// information about the stop place and time
    pub place: crate::place::Place,
    /// Whether there is real-time data about this leg
    pub real_time: bool,
    pub route_color: Option<String>,
    pub route_short_name: String,
    pub route_text_color: Option<String>,
    /// Filename and line number where this trip is from
    pub source: String,
    pub trip_id: String,
}

impl StopTime {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> StopTimeBuilder<
        crate::generics::MissingAgencyId,
        crate::generics::MissingAgencyName,
        crate::generics::MissingAgencyUrl,
        crate::generics::MissingHeadsign,
        crate::generics::MissingMode,
        crate::generics::MissingPlace,
        crate::generics::MissingRealTime,
        crate::generics::MissingRouteShortName,
        crate::generics::MissingSource,
        crate::generics::MissingTripId,
    > {
        StopTimeBuilder {
            body: Default::default(),
            _agency_id: core::marker::PhantomData,
            _agency_name: core::marker::PhantomData,
            _agency_url: core::marker::PhantomData,
            _headsign: core::marker::PhantomData,
            _mode: core::marker::PhantomData,
            _place: core::marker::PhantomData,
            _real_time: core::marker::PhantomData,
            _route_short_name: core::marker::PhantomData,
            _source: core::marker::PhantomData,
            _trip_id: core::marker::PhantomData,
        }
    }
}

impl Into<StopTime>
    for StopTimeBuilder<
        crate::generics::AgencyIdExists,
        crate::generics::AgencyNameExists,
        crate::generics::AgencyUrlExists,
        crate::generics::HeadsignExists,
        crate::generics::ModeExists,
        crate::generics::PlaceExists,
        crate::generics::RealTimeExists,
        crate::generics::RouteShortNameExists,
        crate::generics::SourceExists,
        crate::generics::TripIdExists,
    >
{
    fn into(self) -> StopTime {
        self.body
    }
}

/// Builder for [`StopTime`](./struct.StopTime.html) object.
#[derive(Debug, Clone)]
pub struct StopTimeBuilder<
    AgencyId,
    AgencyName,
    AgencyUrl,
    Headsign,
    Mode,
    Place,
    RealTime,
    RouteShortName,
    Source,
    TripId,
> {
    body: self::StopTime,
    _agency_id: core::marker::PhantomData<AgencyId>,
    _agency_name: core::marker::PhantomData<AgencyName>,
    _agency_url: core::marker::PhantomData<AgencyUrl>,
    _headsign: core::marker::PhantomData<Headsign>,
    _mode: core::marker::PhantomData<Mode>,
    _place: core::marker::PhantomData<Place>,
    _real_time: core::marker::PhantomData<RealTime>,
    _route_short_name: core::marker::PhantomData<RouteShortName>,
    _source: core::marker::PhantomData<Source>,
    _trip_id: core::marker::PhantomData<TripId>,
}

impl<
        AgencyId,
        AgencyName,
        AgencyUrl,
        Headsign,
        Mode,
        Place,
        RealTime,
        RouteShortName,
        Source,
        TripId,
    >
    StopTimeBuilder<
        AgencyId,
        AgencyName,
        AgencyUrl,
        Headsign,
        Mode,
        Place,
        RealTime,
        RouteShortName,
        Source,
        TripId,
    >
{
    #[inline]
    pub fn agency_id(
        mut self,
        value: impl Into<String>,
    ) -> StopTimeBuilder<
        crate::generics::AgencyIdExists,
        AgencyName,
        AgencyUrl,
        Headsign,
        Mode,
        Place,
        RealTime,
        RouteShortName,
        Source,
        TripId,
    > {
        self.body.agency_id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn agency_name(
        mut self,
        value: impl Into<String>,
    ) -> StopTimeBuilder<
        AgencyId,
        crate::generics::AgencyNameExists,
        AgencyUrl,
        Headsign,
        Mode,
        Place,
        RealTime,
        RouteShortName,
        Source,
        TripId,
    > {
        self.body.agency_name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn agency_url(
        mut self,
        value: impl Into<String>,
    ) -> StopTimeBuilder<
        AgencyId,
        AgencyName,
        crate::generics::AgencyUrlExists,
        Headsign,
        Mode,
        Place,
        RealTime,
        RouteShortName,
        Source,
        TripId,
    > {
        self.body.agency_url = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// For transit legs, the headsign of the bus or train being used.
    /// For non-transit legs, null
    ///
    #[inline]
    pub fn headsign(
        mut self,
        value: impl Into<String>,
    ) -> StopTimeBuilder<
        AgencyId,
        AgencyName,
        AgencyUrl,
        crate::generics::HeadsignExists,
        Mode,
        Place,
        RealTime,
        RouteShortName,
        Source,
        TripId,
    > {
        self.body.headsign = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Transport mode for this leg
    #[inline]
    pub fn mode(
        mut self,
        value: crate::mode::Mode,
    ) -> StopTimeBuilder<
        AgencyId,
        AgencyName,
        AgencyUrl,
        Headsign,
        crate::generics::ModeExists,
        Place,
        RealTime,
        RouteShortName,
        Source,
        TripId,
    > {
        self.body.mode = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// information about the stop place and time
    #[inline]
    pub fn place(
        mut self,
        value: crate::place::PlaceBuilder<
            crate::generics::LatExists,
            crate::generics::LevelExists,
            crate::generics::LonExists,
            crate::generics::NameExists,
        >,
    ) -> StopTimeBuilder<
        AgencyId,
        AgencyName,
        AgencyUrl,
        Headsign,
        Mode,
        crate::generics::PlaceExists,
        RealTime,
        RouteShortName,
        Source,
        TripId,
    > {
        self.body.place = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Whether there is real-time data about this leg
    #[inline]
    pub fn real_time(
        mut self,
        value: impl Into<bool>,
    ) -> StopTimeBuilder<
        AgencyId,
        AgencyName,
        AgencyUrl,
        Headsign,
        Mode,
        Place,
        crate::generics::RealTimeExists,
        RouteShortName,
        Source,
        TripId,
    > {
        self.body.real_time = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn route_color(mut self, value: impl Into<String>) -> Self {
        self.body.route_color = Some(value.into());
        self
    }

    #[inline]
    pub fn route_short_name(
        mut self,
        value: impl Into<String>,
    ) -> StopTimeBuilder<
        AgencyId,
        AgencyName,
        AgencyUrl,
        Headsign,
        Mode,
        Place,
        RealTime,
        crate::generics::RouteShortNameExists,
        Source,
        TripId,
    > {
        self.body.route_short_name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn route_text_color(mut self, value: impl Into<String>) -> Self {
        self.body.route_text_color = Some(value.into());
        self
    }

    /// Filename and line number where this trip is from
    #[inline]
    pub fn source(
        mut self,
        value: impl Into<String>,
    ) -> StopTimeBuilder<
        AgencyId,
        AgencyName,
        AgencyUrl,
        Headsign,
        Mode,
        Place,
        RealTime,
        RouteShortName,
        crate::generics::SourceExists,
        TripId,
    > {
        self.body.source = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn trip_id(
        mut self,
        value: impl Into<String>,
    ) -> StopTimeBuilder<
        AgencyId,
        AgencyName,
        AgencyUrl,
        Headsign,
        Mode,
        Place,
        RealTime,
        RouteShortName,
        Source,
        crate::generics::TripIdExists,
    > {
        self.body.trip_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
