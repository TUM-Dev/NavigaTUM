use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepInstruction {
    /// Not implemented!
    /// This step is on an open area, such as a plaza or train platform,
    /// and thus the directions should say something like "cross"
    pub area: bool,
    /// The distance in meters that this step takes.
    pub distance: f64,
    /// Not implemented!
    /// When exiting a highway or traffic circle, the exit name/number.
    pub exit: String,
    /// level where this segment starts, based on OpenStreetMap data
    pub from_level: f64,
    /// OpenStreetMap way index
    pub osm_way: Option<i64>,
    pub polyline: crate::encoded_polyline::EncodedPolyline,
    pub relative_direction: crate::direction::Direction,
    /// Not implemented!
    ///
    /// Indicates whether or not a street changes direction at an intersection.
    pub stay_on: bool,
    /// The name of the street.
    pub street_name: String,
    /// level where this segment starts, based on OpenStreetMap data
    pub to_level: f64,
}

impl StepInstruction {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> StepInstructionBuilder<
        crate::generics::MissingArea,
        crate::generics::MissingDistance,
        crate::generics::MissingExit,
        crate::generics::MissingFromLevel,
        crate::generics::MissingPolyline,
        crate::generics::MissingRelativeDirection,
        crate::generics::MissingStayOn,
        crate::generics::MissingStreetName,
        crate::generics::MissingToLevel,
    > {
        StepInstructionBuilder {
            body: Default::default(),
            _area: core::marker::PhantomData,
            _distance: core::marker::PhantomData,
            _exit: core::marker::PhantomData,
            _from_level: core::marker::PhantomData,
            _polyline: core::marker::PhantomData,
            _relative_direction: core::marker::PhantomData,
            _stay_on: core::marker::PhantomData,
            _street_name: core::marker::PhantomData,
            _to_level: core::marker::PhantomData,
        }
    }
}

impl Into<StepInstruction>
    for StepInstructionBuilder<
        crate::generics::AreaExists,
        crate::generics::DistanceExists,
        crate::generics::ExitExists,
        crate::generics::FromLevelExists,
        crate::generics::PolylineExists,
        crate::generics::RelativeDirectionExists,
        crate::generics::StayOnExists,
        crate::generics::StreetNameExists,
        crate::generics::ToLevelExists,
    >
{
    fn into(self) -> StepInstruction {
        self.body
    }
}

/// Builder for [`StepInstruction`](./struct.StepInstruction.html) object.
#[derive(Debug, Clone)]
pub struct StepInstructionBuilder<
    Area,
    Distance,
    Exit,
    FromLevel,
    Polyline,
    RelativeDirection,
    StayOn,
    StreetName,
    ToLevel,
> {
    body: self::StepInstruction,
    _area: core::marker::PhantomData<Area>,
    _distance: core::marker::PhantomData<Distance>,
    _exit: core::marker::PhantomData<Exit>,
    _from_level: core::marker::PhantomData<FromLevel>,
    _polyline: core::marker::PhantomData<Polyline>,
    _relative_direction: core::marker::PhantomData<RelativeDirection>,
    _stay_on: core::marker::PhantomData<StayOn>,
    _street_name: core::marker::PhantomData<StreetName>,
    _to_level: core::marker::PhantomData<ToLevel>,
}

impl<Area, Distance, Exit, FromLevel, Polyline, RelativeDirection, StayOn, StreetName, ToLevel>
    StepInstructionBuilder<
        Area,
        Distance,
        Exit,
        FromLevel,
        Polyline,
        RelativeDirection,
        StayOn,
        StreetName,
        ToLevel,
    >
{
    /// Not implemented!
    /// This step is on an open area, such as a plaza or train platform,
    /// and thus the directions should say something like "cross"
    ///
    #[inline]
    pub fn area(
        mut self,
        value: impl Into<bool>,
    ) -> StepInstructionBuilder<
        crate::generics::AreaExists,
        Distance,
        Exit,
        FromLevel,
        Polyline,
        RelativeDirection,
        StayOn,
        StreetName,
        ToLevel,
    > {
        self.body.area = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// The distance in meters that this step takes.
    #[inline]
    pub fn distance(
        mut self,
        value: impl Into<f64>,
    ) -> StepInstructionBuilder<
        Area,
        crate::generics::DistanceExists,
        Exit,
        FromLevel,
        Polyline,
        RelativeDirection,
        StayOn,
        StreetName,
        ToLevel,
    > {
        self.body.distance = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Not implemented!
    /// When exiting a highway or traffic circle, the exit name/number.
    ///
    #[inline]
    pub fn exit(
        mut self,
        value: impl Into<String>,
    ) -> StepInstructionBuilder<
        Area,
        Distance,
        crate::generics::ExitExists,
        FromLevel,
        Polyline,
        RelativeDirection,
        StayOn,
        StreetName,
        ToLevel,
    > {
        self.body.exit = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// level where this segment starts, based on OpenStreetMap data
    #[inline]
    pub fn from_level(
        mut self,
        value: impl Into<f64>,
    ) -> StepInstructionBuilder<
        Area,
        Distance,
        Exit,
        crate::generics::FromLevelExists,
        Polyline,
        RelativeDirection,
        StayOn,
        StreetName,
        ToLevel,
    > {
        self.body.from_level = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// OpenStreetMap way index
    #[inline]
    pub fn osm_way(mut self, value: impl Into<i64>) -> Self {
        self.body.osm_way = Some(value.into());
        self
    }

    #[inline]
    pub fn polyline(
        mut self,
        value: crate::encoded_polyline::EncodedPolylineBuilder<
            crate::generics::LengthExists,
            crate::generics::PointsExists,
        >,
    ) -> StepInstructionBuilder<
        Area,
        Distance,
        Exit,
        FromLevel,
        crate::generics::PolylineExists,
        RelativeDirection,
        StayOn,
        StreetName,
        ToLevel,
    > {
        self.body.polyline = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn relative_direction(
        mut self,
        value: crate::direction::Direction,
    ) -> StepInstructionBuilder<
        Area,
        Distance,
        Exit,
        FromLevel,
        Polyline,
        crate::generics::RelativeDirectionExists,
        StayOn,
        StreetName,
        ToLevel,
    > {
        self.body.relative_direction = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Not implemented!
    /// Indicates whether or not a street changes direction at an intersection.
    ///
    #[inline]
    pub fn stay_on(
        mut self,
        value: impl Into<bool>,
    ) -> StepInstructionBuilder<
        Area,
        Distance,
        Exit,
        FromLevel,
        Polyline,
        RelativeDirection,
        crate::generics::StayOnExists,
        StreetName,
        ToLevel,
    > {
        self.body.stay_on = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// The name of the street.
    #[inline]
    pub fn street_name(
        mut self,
        value: impl Into<String>,
    ) -> StepInstructionBuilder<
        Area,
        Distance,
        Exit,
        FromLevel,
        Polyline,
        RelativeDirection,
        StayOn,
        crate::generics::StreetNameExists,
        ToLevel,
    > {
        self.body.street_name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// level where this segment starts, based on OpenStreetMap data
    #[inline]
    pub fn to_level(
        mut self,
        value: impl Into<f64>,
    ) -> StepInstructionBuilder<
        Area,
        Distance,
        Exit,
        FromLevel,
        Polyline,
        RelativeDirection,
        StayOn,
        StreetName,
        crate::generics::ToLevelExists,
    > {
        self.body.to_level = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
