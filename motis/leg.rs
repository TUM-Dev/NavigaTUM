use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leg {
    pub agency_id: Option<String>,
    pub agency_name: Option<String>,
    pub agency_url: Option<String>,
    /// For non-transit legs the distance traveled while traversing this leg in meters.
    pub distance: Option<f64>,
    /// Leg duration in seconds
    ///
    /// If leg is footpath:
    /// The footpath duration is derived from the default footpath duration.
    /// The query parameters `transferTimeFactor` and  `additionalTransferTime` are used as follows:
    /// - `leg.duration = defaultDuration * transferTimeFactor + additionalTransferTime.`
    /// 
    /// In case the `defaultDuration` is needed, it can be calculated by
    /// - `defaultDuration = (leg.duration - additionalTransferTime) / transferTimeFactor`.
    /// 
    /// Note that the default (if not explicitly provided in the query) values are
    /// - `transferTimeFactor = 1` and
    /// - `additionalTransferTime = 0`
    pub duration: i64,
    /// leg arrival time
    pub end_time: String,
    pub from: crate::place::Place,
    /// For transit legs, the headsign of the bus or train being used.
    /// [`None`] for non-transit legs.
    pub headsign: Option<String>,
    /// For transit legs, if the rider should stay on the vehicle as it changes route names.
    pub interline_with_previous_leg: Option<bool>,
    /// For transit legs, intermediate stops between the Place where the leg originates
    /// and the Place where the leg ends.
    /// 
    /// [`None`] for non-transit legs.
    pub intermediate_stops: Option<Vec<crate::place::Place>>,
    pub leg_geometry: crate::encoded_polyline::EncodedPolyline,
    /// Transport mode for this leg
    pub mode: crate::mode::Mode,
    /// Whether there is real-time data about this leg
    pub real_time: bool,
    pub rental: Option<crate::rental::Rental>,
    pub route_color: Option<String>,
    pub route_short_name: Option<String>,
    pub route_text_color: Option<String>,
    pub route_type: Option<String>,
    /// scheduled leg arrival time
    pub scheduled_end_time: String,
    /// scheduled leg departure time
    pub scheduled_start_time: String,
    /// Filename and line number where this trip is from
    pub source: Option<String>,
    /// leg departure time
    pub start_time: String,
    /// A series of turn by turn instructions
    /// used for walking, biking and driving.
    pub steps: Option<Vec<crate::step_instruction::StepInstruction>>,
    pub to: crate::place::Place,
    pub trip_id: Option<String>,
}

impl Leg {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> LegBuilder<
        crate::generics::MissingDuration,
        crate::generics::MissingEndTime,
        crate::generics::MissingFrom,
        crate::generics::MissingLegGeometry,
        crate::generics::MissingMode,
        crate::generics::MissingRealTime,
        crate::generics::MissingScheduledEndTime,
        crate::generics::MissingScheduledStartTime,
        crate::generics::MissingStartTime,
        crate::generics::MissingTo,
    > {
        LegBuilder {
            body: Default::default(),
            _duration: core::marker::PhantomData,
            _end_time: core::marker::PhantomData,
            _from: core::marker::PhantomData,
            _leg_geometry: core::marker::PhantomData,
            _mode: core::marker::PhantomData,
            _real_time: core::marker::PhantomData,
            _scheduled_end_time: core::marker::PhantomData,
            _scheduled_start_time: core::marker::PhantomData,
            _start_time: core::marker::PhantomData,
            _to: core::marker::PhantomData,
        }
    }
}

impl Into<Leg>
    for LegBuilder<
        crate::generics::DurationExists,
        crate::generics::EndTimeExists,
        crate::generics::FromExists,
        crate::generics::LegGeometryExists,
        crate::generics::ModeExists,
        crate::generics::RealTimeExists,
        crate::generics::ScheduledEndTimeExists,
        crate::generics::ScheduledStartTimeExists,
        crate::generics::StartTimeExists,
        crate::generics::ToExists,
    >
{
    fn into(self) -> Leg {
        self.body
    }
}

/// Builder for [`Leg`](./struct.Leg.html) object.
#[derive(Debug, Clone)]
pub struct LegBuilder<
    Duration,
    EndTime,
    From,
    LegGeometry,
    Mode,
    RealTime,
    ScheduledEndTime,
    ScheduledStartTime,
    StartTime,
    To,
> {
    body: self::Leg,
    _duration: core::marker::PhantomData<Duration>,
    _end_time: core::marker::PhantomData<EndTime>,
    _from: core::marker::PhantomData<From>,
    _leg_geometry: core::marker::PhantomData<LegGeometry>,
    _mode: core::marker::PhantomData<Mode>,
    _real_time: core::marker::PhantomData<RealTime>,
    _scheduled_end_time: core::marker::PhantomData<ScheduledEndTime>,
    _scheduled_start_time: core::marker::PhantomData<ScheduledStartTime>,
    _start_time: core::marker::PhantomData<StartTime>,
    _to: core::marker::PhantomData<To>,
}

impl<
        Duration,
        EndTime,
        From,
        LegGeometry,
        Mode,
        RealTime,
        ScheduledEndTime,
        ScheduledStartTime,
        StartTime,
        To,
    >
    LegBuilder<
        Duration,
        EndTime,
        From,
        LegGeometry,
        Mode,
        RealTime,
        ScheduledEndTime,
        ScheduledStartTime,
        StartTime,
        To,
    >
{
    #[inline]
    pub fn agency_id(mut self, value: impl Into<String>) -> Self {
        self.body.agency_id = Some(value.into());
        self
    }

    #[inline]
    pub fn agency_name(mut self, value: impl Into<String>) -> Self {
        self.body.agency_name = Some(value.into());
        self
    }

    #[inline]
    pub fn agency_url(mut self, value: impl Into<String>) -> Self {
        self.body.agency_url = Some(value.into());
        self
    }

    /// For non-transit legs the distance traveled while traversing this leg in meters.
    #[inline]
    pub fn distance(mut self, value: impl Into<f64>) -> Self {
        self.body.distance = Some(value.into());
        self
    }

    /// Leg duration in seconds
    ///
    /// If leg is footpath:
    ///   The footpath duration is derived from the default footpath
    ///   duration using the query parameters `transferTimeFactor` and
    ///   `additionalTransferTime` as follows:
    ///   `leg.duration = defaultDuration * transferTimeFactor + additionalTransferTime.`
    ///   In case the defaultDuration is needed, it can be calculated by
    ///   `defaultDuration = (leg.duration - additionalTransferTime) / transferTimeFactor`.
    ///   Note that the default values are `transferTimeFactor = 1` and
    ///   `additionalTransferTime = 0` in case they are not explicitly
    ///   provided in the query.
    ///
    #[inline]
    pub fn duration(
        mut self,
        value: impl Into<i64>,
    ) -> LegBuilder<
        crate::generics::DurationExists,
        EndTime,
        From,
        LegGeometry,
        Mode,
        RealTime,
        ScheduledEndTime,
        ScheduledStartTime,
        StartTime,
        To,
    > {
        self.body.duration = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// leg arrival time
    #[inline]
    pub fn end_time(
        mut self,
        value: impl Into<String>,
    ) -> LegBuilder<
        Duration,
        crate::generics::EndTimeExists,
        From,
        LegGeometry,
        Mode,
        RealTime,
        ScheduledEndTime,
        ScheduledStartTime,
        StartTime,
        To,
    > {
        self.body.end_time = value.into();
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
    ) -> LegBuilder<
        Duration,
        EndTime,
        crate::generics::FromExists,
        LegGeometry,
        Mode,
        RealTime,
        ScheduledEndTime,
        ScheduledStartTime,
        StartTime,
        To,
    > {
        self.body.from = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// For transit legs, the headsign of the bus or train being used.
    /// For non-transit legs, null
    ///
    #[inline]
    pub fn headsign(mut self, value: impl Into<String>) -> Self {
        self.body.headsign = Some(value.into());
        self
    }

    /// For transit legs, if the rider should stay on the vehicle as it changes route names.
    #[inline]
    pub fn interline_with_previous_leg(mut self, value: impl Into<bool>) -> Self {
        self.body.interline_with_previous_leg = Some(value.into());
        self
    }

    /// For transit legs, intermediate stops between the Place where the leg originates
    /// and the Place where the leg ends.
    /// 
    /// [`None`] for non-transit legs.
    #[inline]
    pub fn intermediate_stops(
        mut self,
        value: impl Iterator<
            Item = crate::place::PlaceBuilder<
                crate::generics::LatExists,
                crate::generics::LevelExists,
                crate::generics::LonExists,
                crate::generics::NameExists,
            >,
        >,
    ) -> Self {
        self.body.intermediate_stops =
            Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn leg_geometry(
        mut self,
        value: crate::encoded_polyline::EncodedPolylineBuilder<
            crate::generics::LengthExists,
            crate::generics::PointsExists,
        >,
    ) -> LegBuilder<
        Duration,
        EndTime,
        From,
        crate::generics::LegGeometryExists,
        Mode,
        RealTime,
        ScheduledEndTime,
        ScheduledStartTime,
        StartTime,
        To,
    > {
        self.body.leg_geometry = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Transport mode for this leg
    #[inline]
    pub fn mode(
        mut self,
        value: crate::mode::Mode,
    ) -> LegBuilder<
        Duration,
        EndTime,
        From,
        LegGeometry,
        crate::generics::ModeExists,
        RealTime,
        ScheduledEndTime,
        ScheduledStartTime,
        StartTime,
        To,
    > {
        self.body.mode = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Whether there is real-time data about this leg
    #[inline]
    pub fn real_time(
        mut self,
        value: impl Into<bool>,
    ) -> LegBuilder<
        Duration,
        EndTime,
        From,
        LegGeometry,
        Mode,
        crate::generics::RealTimeExists,
        ScheduledEndTime,
        ScheduledStartTime,
        StartTime,
        To,
    > {
        self.body.real_time = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn rental(
        mut self,
        value: crate::rental::RentalBuilder<crate::generics::SystemIdExists>,
    ) -> Self {
        self.body.rental = Some(value.into());
        self
    }

    #[inline]
    pub fn route_color(mut self, value: impl Into<String>) -> Self {
        self.body.route_color = Some(value.into());
        self
    }

    #[inline]
    pub fn route_short_name(mut self, value: impl Into<String>) -> Self {
        self.body.route_short_name = Some(value.into());
        self
    }

    #[inline]
    pub fn route_text_color(mut self, value: impl Into<String>) -> Self {
        self.body.route_text_color = Some(value.into());
        self
    }

    #[inline]
    pub fn route_type(mut self, value: impl Into<String>) -> Self {
        self.body.route_type = Some(value.into());
        self
    }

    /// scheduled leg arrival time
    #[inline]
    pub fn scheduled_end_time(
        mut self,
        value: impl Into<String>,
    ) -> LegBuilder<
        Duration,
        EndTime,
        From,
        LegGeometry,
        Mode,
        RealTime,
        crate::generics::ScheduledEndTimeExists,
        ScheduledStartTime,
        StartTime,
        To,
    > {
        self.body.scheduled_end_time = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// scheduled leg departure time
    #[inline]
    pub fn scheduled_start_time(
        mut self,
        value: impl Into<String>,
    ) -> LegBuilder<
        Duration,
        EndTime,
        From,
        LegGeometry,
        Mode,
        RealTime,
        ScheduledEndTime,
        crate::generics::ScheduledStartTimeExists,
        StartTime,
        To,
    > {
        self.body.scheduled_start_time = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Filename and line number where this trip is from
    #[inline]
    pub fn source(mut self, value: impl Into<String>) -> Self {
        self.body.source = Some(value.into());
        self
    }

    /// leg departure time
    #[inline]
    pub fn start_time(
        mut self,
        value: impl Into<String>,
    ) -> LegBuilder<
        Duration,
        EndTime,
        From,
        LegGeometry,
        Mode,
        RealTime,
        ScheduledEndTime,
        ScheduledStartTime,
        crate::generics::StartTimeExists,
        To,
    > {
        self.body.start_time = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// A series of turn by turn instructions
    /// used for walking, biking and driving.
    ///
    #[inline]
    pub fn steps(
        mut self,
        value: impl Iterator<
            Item = crate::step_instruction::StepInstructionBuilder<
                crate::generics::AreaExists,
                crate::generics::DistanceExists,
                crate::generics::ExitExists,
                crate::generics::FromLevelExists,
                crate::generics::PolylineExists,
                crate::generics::RelativeDirectionExists,
                crate::generics::StayOnExists,
                crate::generics::StreetNameExists,
                crate::generics::ToLevelExists,
            >,
        >,
    ) -> Self {
        self.body.steps = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
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
    ) -> LegBuilder<
        Duration,
        EndTime,
        From,
        LegGeometry,
        Mode,
        RealTime,
        ScheduledEndTime,
        ScheduledStartTime,
        StartTime,
        crate::generics::ToExists,
    > {
        self.body.to = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn trip_id(mut self, value: impl Into<String>) -> Self {
        self.body.trip_id = Some(value.into());
        self
    }
}
