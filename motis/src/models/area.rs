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
    pub fn builder() -> Area {
        Area::default()
    }

    /// [OpenStreetMap `admin_level`](https://wiki.openstreetmap.org/wiki/Key:admin_level)
    /// of the area
    #[inline]
    pub fn admin_level(mut self, value: f64) -> Area {
        self.admin_level = value;
        self
    }

    /// Whether this area should be displayed as default area (area with admin level closest 7)
    #[inline]
    pub fn displayed_as_default(mut self, value: bool) -> Self {
        self.default = Some(value);
        self
    }

    /// Whether this area was matched by the input text
    #[inline]
    pub fn matched(mut self, value: bool) -> Area {
        self.matched = value;
        self
    }

    /// Name of the area
    #[inline]
    pub fn name(mut self, value: impl ToString) -> Area {
        self.name = value.to_string();
        self
    }
}
