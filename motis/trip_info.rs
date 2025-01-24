use serde::{Deserialize, Serialize};

/// trip id and name
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripInfo {
    /// trip display name
    pub route_short_name: String,
    /// trip ID (dataset trip id prefixed with the dataset tag)
    pub trip_id: String,
}

impl TripInfo {
    /// Create a builder for this object.
    #[inline]
    pub fn builder(
    ) -> TripInfoBuilder<crate::generics::MissingRouteShortName, crate::generics::MissingTripId>
    {
        TripInfoBuilder {
            body: Default::default(),
            _route_short_name: core::marker::PhantomData,
            _trip_id: core::marker::PhantomData,
        }
    }
}

impl Into<TripInfo>
    for TripInfoBuilder<crate::generics::RouteShortNameExists, crate::generics::TripIdExists>
{
    fn into(self) -> TripInfo {
        self.body
    }
}

/// Builder for [`TripInfo`](./struct.TripInfo.html) object.
#[derive(Debug, Clone)]
pub struct TripInfoBuilder<RouteShortName, TripId> {
    body: self::TripInfo,
    _route_short_name: core::marker::PhantomData<RouteShortName>,
    _trip_id: core::marker::PhantomData<TripId>,
}

impl<RouteShortName, TripId> TripInfoBuilder<RouteShortName, TripId> {
    /// trip display name
    #[inline]
    pub fn route_short_name(
        mut self,
        value: impl Into<String>,
    ) -> TripInfoBuilder<crate::generics::RouteShortNameExists, TripId> {
        self.body.route_short_name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// trip ID (dataset trip id prefixed with the dataset tag)
    #[inline]
    pub fn trip_id(
        mut self,
        value: impl Into<String>,
    ) -> TripInfoBuilder<RouteShortName, crate::generics::TripIdExists> {
        self.body.trip_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
