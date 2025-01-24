use serde::{Deserialize, Serialize};

/// Administrative area
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Area {
    /// [OpenStreetMap `admin_level`](https://wiki.openstreetmap.org/wiki/Key:admin_level)
    /// of the area
    pub admin_level: f64,
    /// Whether this area should be displayed as default area (area with admin level closest 7)
    pub default: Option<bool>,
    /// Whether this area was matched by the input text
    pub matched: bool,
    /// Name of the area
    pub name: String,
}

impl Area {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> AreaBuilder<
        crate::generics::MissingAdminLevel,
        crate::generics::MissingMatched,
        crate::generics::MissingName,
    > {
        AreaBuilder {
            body: Default::default(),
            _admin_level: core::marker::PhantomData,
            _matched: core::marker::PhantomData,
            _name: core::marker::PhantomData,
        }
    }
}

impl Into<Area>
    for AreaBuilder<
        crate::generics::AdminLevelExists,
        crate::generics::MatchedExists,
        crate::generics::NameExists,
    >
{
    fn into(self) -> Area {
        self.body
    }
}

/// Builder for [`Area`](./struct.Area.html) object.
#[derive(Debug, Clone)]
pub struct AreaBuilder<AdminLevel, Matched, Name> {
    body: self::Area,
    _admin_level: core::marker::PhantomData<AdminLevel>,
    _matched: core::marker::PhantomData<Matched>,
    _name: core::marker::PhantomData<Name>,
}

impl<AdminLevel, Matched, Name> AreaBuilder<AdminLevel, Matched, Name> {
    /// [OpenStreetMap `admin_level`](https://wiki.openstreetmap.org/wiki/Key:admin_level)
    /// of the area
    ///
    #[inline]
    pub fn admin_level(
        mut self,
        value: impl Into<f64>,
    ) -> AreaBuilder<crate::generics::AdminLevelExists, Matched, Name> {
        self.body.admin_level = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Whether this area should be displayed as default area (area with admin level closest 7)
    #[inline]
    pub fn default(mut self, value: impl Into<bool>) -> Self {
        self.body.default = Some(value.into());
        self
    }

    /// Whether this area was matched by the input text
    #[inline]
    pub fn matched(
        mut self,
        value: impl Into<bool>,
    ) -> AreaBuilder<AdminLevel, crate::generics::MatchedExists, Name> {
        self.body.matched = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Name of the area
    #[inline]
    pub fn name(
        mut self,
        value: impl Into<String>,
    ) -> AreaBuilder<AdminLevel, Matched, crate::generics::NameExists> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
