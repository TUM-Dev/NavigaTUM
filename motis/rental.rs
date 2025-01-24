use serde::{Deserialize, Serialize};

/// Vehicle rental
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rental {
    pub form_factor: Option<crate::rental_form_factor::RentalFormFactor>,
    pub propulsion_type: Option<crate::rental_propulsion_type::RentalPropulsionType>,
    /// Rental URI for Android (deep link to the specific station or vehicle)
    pub rental_uri_android: Option<String>,
    /// Rental URI for iOS (deep link to the specific station or vehicle)
    pub rental_uri_ios: Option<String>,
    /// Rental URI for web (deep link to the specific station or vehicle)
    pub rental_uri_web: Option<String>,
    pub return_constraint: Option<crate::rental_return_constraint::RentalReturnConstraint>,
    /// Name of the station
    pub station_name: Option<String>,
    /// Vehicle share system ID
    pub system_id: String,
    /// Vehicle share system name
    pub system_name: Option<String>,
    /// URL of the vehicle share system
    pub url: Option<String>,
}

impl Rental {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RentalBuilder<crate::generics::MissingSystemId> {
        RentalBuilder {
            body: Default::default(),
            _system_id: core::marker::PhantomData,
        }
    }
}

impl Into<Rental> for RentalBuilder<crate::generics::SystemIdExists> {
    fn into(self) -> Rental {
        self.body
    }
}

/// Builder for [`Rental`](./struct.Rental.html) object.
#[derive(Debug, Clone)]
pub struct RentalBuilder<SystemId> {
    body: self::Rental,
    _system_id: core::marker::PhantomData<SystemId>,
}

impl<SystemId> RentalBuilder<SystemId> {
    #[inline]
    pub fn form_factor(mut self, value: crate::rental_form_factor::RentalFormFactor) -> Self {
        self.body.form_factor = Some(value.into());
        self
    }

    #[inline]
    pub fn propulsion_type(
        mut self,
        value: crate::rental_propulsion_type::RentalPropulsionType,
    ) -> Self {
        self.body.propulsion_type = Some(value.into());
        self
    }

    /// Rental URI for Android (deep link to the specific station or vehicle)
    #[inline]
    pub fn rental_uri_android(mut self, value: impl Into<String>) -> Self {
        self.body.rental_uri_android = Some(value.into());
        self
    }

    /// Rental URI for iOS (deep link to the specific station or vehicle)
    #[inline]
    pub fn rental_uri_ios(mut self, value: impl Into<String>) -> Self {
        self.body.rental_uri_ios = Some(value.into());
        self
    }

    /// Rental URI for web (deep link to the specific station or vehicle)
    #[inline]
    pub fn rental_uri_web(mut self, value: impl Into<String>) -> Self {
        self.body.rental_uri_web = Some(value.into());
        self
    }

    #[inline]
    pub fn return_constraint(
        mut self,
        value: crate::rental_return_constraint::RentalReturnConstraint,
    ) -> Self {
        self.body.return_constraint = Some(value.into());
        self
    }

    /// Name of the station
    #[inline]
    pub fn station_name(mut self, value: impl Into<String>) -> Self {
        self.body.station_name = Some(value.into());
        self
    }

    /// Vehicle share system ID
    #[inline]
    pub fn system_id(
        mut self,
        value: impl Into<String>,
    ) -> RentalBuilder<crate::generics::SystemIdExists> {
        self.body.system_id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Vehicle share system name
    #[inline]
    pub fn system_name(mut self, value: impl Into<String>) -> Self {
        self.body.system_name = Some(value.into());
        self
    }

    /// URL of the vehicle share system
    #[inline]
    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.body.url = Some(value.into());
        self
    }
}
