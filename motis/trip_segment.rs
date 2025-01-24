use serde::{Deserialize, Serialize};

/// trip segment between two stops to show a trip on a map
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TripSegment {
    /// arrival time
    pub arrival: String,
    /// departure time
    pub departure: String,
    /// distance in meters
    pub distance: f64,
    pub from: crate::place::Place,
    /// Transport mode for this leg
    pub mode: crate::mode::Mode,
    /// Google polyline encoded coordinate sequence (with precision 7) where the trip travels on this segment.
    pub polyline: String,
    /// Whether there is real-time data about this leg
    #[serde(rename = "realTime")]
    pub real_time: bool,
    #[serde(rename = "routeColor")]
    pub route_color: Option<String>,
    /// scheduled arrival time
    #[serde(rename = "scheduledArrival")]
    pub scheduled_arrival: String,
    /// scheduled departure time
    #[serde(rename = "scheduledDeparture")]
    pub scheduled_departure: String,
    pub to: crate::place::Place,
    pub trips: Vec<crate::trip_info::TripInfo>,
}

impl TripSegment {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> TripSegmentBuilder<
        crate::generics::MissingArrival,
        crate::generics::MissingDeparture,
        crate::generics::MissingDistance,
        crate::generics::MissingFrom,
        crate::generics::MissingMode,
        crate::generics::MissingPolyline,
        crate::generics::MissingRealTime,
        crate::generics::MissingScheduledArrival,
        crate::generics::MissingScheduledDeparture,
        crate::generics::MissingTo,
        crate::generics::MissingTrips,
    > {
        TripSegmentBuilder {
            body: Default::default(),
            _arrival: core::marker::PhantomData,
            _departure: core::marker::PhantomData,
            _distance: core::marker::PhantomData,
            _from: core::marker::PhantomData,
            _mode: core::marker::PhantomData,
            _polyline: core::marker::PhantomData,
            _real_time: core::marker::PhantomData,
            _scheduled_arrival: core::marker::PhantomData,
            _scheduled_departure: core::marker::PhantomData,
            _to: core::marker::PhantomData,
            _trips: core::marker::PhantomData,
        }
    }
}

impl Into<TripSegment>
    for TripSegmentBuilder<
        crate::generics::ArrivalExists,
        crate::generics::DepartureExists,
        crate::generics::DistanceExists,
        crate::generics::FromExists,
        crate::generics::ModeExists,
        crate::generics::PolylineExists,
        crate::generics::RealTimeExists,
        crate::generics::ScheduledArrivalExists,
        crate::generics::ScheduledDepartureExists,
        crate::generics::ToExists,
        crate::generics::TripsExists,
    >
{
    fn into(self) -> TripSegment {
        self.body
    }
}

/// Builder for [`TripSegment`](./struct.TripSegment.html) object.
#[derive(Debug, Clone)]
pub struct TripSegmentBuilder<
    Arrival,
    Departure,
    Distance,
    From,
    Mode,
    Polyline,
    RealTime,
    ScheduledArrival,
    ScheduledDeparture,
    To,
    Trips,
> {
    body: self::TripSegment,
    _arrival: core::marker::PhantomData<Arrival>,
    _departure: core::marker::PhantomData<Departure>,
    _distance: core::marker::PhantomData<Distance>,
    _from: core::marker::PhantomData<From>,
    _mode: core::marker::PhantomData<Mode>,
    _polyline: core::marker::PhantomData<Polyline>,
    _real_time: core::marker::PhantomData<RealTime>,
    _scheduled_arrival: core::marker::PhantomData<ScheduledArrival>,
    _scheduled_departure: core::marker::PhantomData<ScheduledDeparture>,
    _to: core::marker::PhantomData<To>,
    _trips: core::marker::PhantomData<Trips>,
}

impl<
        Arrival,
        Departure,
        Distance,
        From,
        Mode,
        Polyline,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        Trips,
    >
    TripSegmentBuilder<
        Arrival,
        Departure,
        Distance,
        From,
        Mode,
        Polyline,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        Trips,
    >
{
    /// arrival time
    #[inline]
    pub fn arrival(
        mut self,
        value: impl Into<String>,
    ) -> TripSegmentBuilder<
        crate::generics::ArrivalExists,
        Departure,
        Distance,
        From,
        Mode,
        Polyline,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        Trips,
    > {
        self.body.arrival = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// departure time
    #[inline]
    pub fn departure(
        mut self,
        value: impl Into<String>,
    ) -> TripSegmentBuilder<
        Arrival,
        crate::generics::DepartureExists,
        Distance,
        From,
        Mode,
        Polyline,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        Trips,
    > {
        self.body.departure = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// distance in meters
    #[inline]
    pub fn distance(
        mut self,
        value: impl Into<f64>,
    ) -> TripSegmentBuilder<
        Arrival,
        Departure,
        crate::generics::DistanceExists,
        From,
        Mode,
        Polyline,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        Trips,
    > {
        self.body.distance = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn from(
        mut self,
        value: crate::place::PlaceBuilder<
            crate::generics::LatExists,
            crate::generics::LevelExists,
            crate::generics::LonExists,
            crate::generics::NameExists,
        >,
    ) -> TripSegmentBuilder<
        Arrival,
        Departure,
        Distance,
        crate::generics::FromExists,
        Mode,
        Polyline,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        Trips,
    > {
        self.body.from = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Transport mode for this leg
    #[inline]
    pub fn mode(
        mut self,
        value: crate::mode::Mode,
    ) -> TripSegmentBuilder<
        Arrival,
        Departure,
        Distance,
        From,
        crate::generics::ModeExists,
        Polyline,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        Trips,
    > {
        self.body.mode = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Google polyline encoded coordinate sequence (with precision 7) where the trip travels on this segment.
    #[inline]
    pub fn polyline(
        mut self,
        value: impl Into<String>,
    ) -> TripSegmentBuilder<
        Arrival,
        Departure,
        Distance,
        From,
        Mode,
        crate::generics::PolylineExists,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        Trips,
    > {
        self.body.polyline = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Whether there is real-time data about this leg
    #[inline]
    pub fn real_time(
        mut self,
        value: impl Into<bool>,
    ) -> TripSegmentBuilder<
        Arrival,
        Departure,
        Distance,
        From,
        Mode,
        Polyline,
        crate::generics::RealTimeExists,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        Trips,
    > {
        self.body.real_time = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn route_color(mut self, value: impl Into<String>) -> Self {
        self.body.route_color = Some(value.into());
        self
    }

    /// scheduled arrival time
    #[inline]
    pub fn scheduled_arrival(
        mut self,
        value: impl Into<String>,
    ) -> TripSegmentBuilder<
        Arrival,
        Departure,
        Distance,
        From,
        Mode,
        Polyline,
        RealTime,
        crate::generics::ScheduledArrivalExists,
        ScheduledDeparture,
        To,
        Trips,
    > {
        self.body.scheduled_arrival = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// scheduled departure time
    #[inline]
    pub fn scheduled_departure(
        mut self,
        value: impl Into<String>,
    ) -> TripSegmentBuilder<
        Arrival,
        Departure,
        Distance,
        From,
        Mode,
        Polyline,
        RealTime,
        ScheduledArrival,
        crate::generics::ScheduledDepartureExists,
        To,
        Trips,
    > {
        self.body.scheduled_departure = value.into();
        unsafe { std::mem::transmute(self) }
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
    ) -> TripSegmentBuilder<
        Arrival,
        Departure,
        Distance,
        From,
        Mode,
        Polyline,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        crate::generics::ToExists,
        Trips,
    > {
        self.body.to = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn trips(
        mut self,
        value: impl Iterator<
            Item = crate::trip_info::TripInfoBuilder<
                crate::generics::RouteShortNameExists,
                crate::generics::TripIdExists,
            >,
        >,
    ) -> TripSegmentBuilder<
        Arrival,
        Departure,
        Distance,
        From,
        Mode,
        Polyline,
        RealTime,
        ScheduledArrival,
        ScheduledDeparture,
        To,
        crate::generics::TripsExists,
    > {
        self.body.trips = value.map(|value| value.into()).collect::<Vec<_>>().into();
        unsafe { std::mem::transmute(self) }
    }
}
