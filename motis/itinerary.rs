use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Itinerary {
    /// journey duration in seconds
    pub duration: i64,
    /// journey arrival time
    pub end_time: String,
    /// Journey legs
    pub legs: Vec<crate::leg::Leg>,
    /// journey departure time
    pub start_time: String,
    /// The number of transfers this trip has.
    pub transfers: i64,
}

impl Itinerary {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ItineraryBuilder<
        crate::generics::MissingDuration,
        crate::generics::MissingEndTime,
        crate::generics::MissingLegs,
        crate::generics::MissingStartTime,
        crate::generics::MissingTransfers,
    > {
        ItineraryBuilder {
            body: Default::default(),
            _duration: core::marker::PhantomData,
            _end_time: core::marker::PhantomData,
            _legs: core::marker::PhantomData,
            _start_time: core::marker::PhantomData,
            _transfers: core::marker::PhantomData,
        }
    }
}

impl Into<Itinerary>
    for ItineraryBuilder<
        crate::generics::DurationExists,
        crate::generics::EndTimeExists,
        crate::generics::LegsExists,
        crate::generics::StartTimeExists,
        crate::generics::TransfersExists,
    >
{
    fn into(self) -> Itinerary {
        self.body
    }
}

/// Builder for [`Itinerary`](./struct.Itinerary.html) object.
#[derive(Debug, Clone)]
pub struct ItineraryBuilder<Duration, EndTime, Legs, StartTime, Transfers> {
    body: self::Itinerary,
    _duration: core::marker::PhantomData<Duration>,
    _end_time: core::marker::PhantomData<EndTime>,
    _legs: core::marker::PhantomData<Legs>,
    _start_time: core::marker::PhantomData<StartTime>,
    _transfers: core::marker::PhantomData<Transfers>,
}

impl<Duration, EndTime, Legs, StartTime, Transfers>
    ItineraryBuilder<Duration, EndTime, Legs, StartTime, Transfers>
{
    /// journey duration in seconds
    #[inline]
    pub fn duration(
        mut self,
        value: impl Into<i64>,
    ) -> ItineraryBuilder<crate::generics::DurationExists, EndTime, Legs, StartTime, Transfers>
    {
        self.body.duration = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// journey arrival time
    #[inline]
    pub fn end_time(
        mut self,
        value: impl Into<String>,
    ) -> ItineraryBuilder<Duration, crate::generics::EndTimeExists, Legs, StartTime, Transfers>
    {
        self.body.end_time = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Journey legs
    #[inline]
    pub fn legs(
        mut self,
        value: impl Iterator<
            Item = crate::leg::LegBuilder<
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
            >,
        >,
    ) -> ItineraryBuilder<Duration, EndTime, crate::generics::LegsExists, StartTime, Transfers>
    {
        self.body.legs = value.map(|value| value.into()).collect::<Vec<_>>().into();
        unsafe { std::mem::transmute(self) }
    }

    /// journey departure time
    #[inline]
    pub fn start_time(
        mut self,
        value: impl Into<String>,
    ) -> ItineraryBuilder<Duration, EndTime, Legs, crate::generics::StartTimeExists, Transfers>
    {
        self.body.start_time = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// The number of transfers this trip has.
    #[inline]
    pub fn transfers(
        mut self,
        value: impl Into<i64>,
    ) -> ItineraryBuilder<Duration, EndTime, Legs, StartTime, crate::generics::TransfersExists>
    {
        self.body.transfers = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
