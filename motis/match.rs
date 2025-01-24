use serde::{Deserialize, Serialize};

/// GeoCoding match
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Match {
    /// list of areas
    pub areas: Vec<crate::area::Area>,
    /// house number
    #[serde(rename = "houseNumber")]
    pub house_number: Option<String>,
    /// unique ID of the location
    pub id: String,
    /// latitude
    pub lat: f64,
    /// level according to OpenStreetMap
    /// (at the moment only for public transport)
    pub level: Option<f64>,
    /// longitude
    pub lon: f64,
    /// name of the location (transit stop / PoI / address)
    pub name: String,
    /// score according to the internal scoring system (the scoring algorithm might change in the future)
    pub score: f64,
    /// street name
    pub street: Option<String>,
    /// list of non-overlapping tokens that were matched
    pub tokens: Vec<Vec<f64>>,
    /// location type
    #[serde(rename = "type")]
    pub type_: crate::r#match::MatchType,
    /// zip code
    pub zip: Option<String>,
}

/// location type
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchType {
    #[default]
    Address,
    Place,
    Stop,
}

impl Match {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> MatchBuilder<
        crate::generics::MissingAreas,
        crate::generics::MissingId,
        crate::generics::MissingLat,
        crate::generics::MissingLon,
        crate::generics::MissingName,
        crate::generics::MissingScore,
        crate::generics::MissingTokens,
        crate::generics::MissingType,
    > {
        MatchBuilder {
            body: Default::default(),
            _areas: core::marker::PhantomData,
            _id: core::marker::PhantomData,
            _lat: core::marker::PhantomData,
            _lon: core::marker::PhantomData,
            _name: core::marker::PhantomData,
            _score: core::marker::PhantomData,
            _tokens: core::marker::PhantomData,
            _type: core::marker::PhantomData,
        }
    }
}

impl Into<Match>
    for MatchBuilder<
        crate::generics::AreasExists,
        crate::generics::IdExists,
        crate::generics::LatExists,
        crate::generics::LonExists,
        crate::generics::NameExists,
        crate::generics::ScoreExists,
        crate::generics::TokensExists,
        crate::generics::TypeExists,
    >
{
    fn into(self) -> Match {
        self.body
    }
}

/// Builder for [`Match`](./struct.Match.html) object.
#[derive(Debug, Clone)]
pub struct MatchBuilder<Areas, Id, Lat, Lon, Name, Score, Tokens, Type> {
    body: self::Match,
    _areas: core::marker::PhantomData<Areas>,
    _id: core::marker::PhantomData<Id>,
    _lat: core::marker::PhantomData<Lat>,
    _lon: core::marker::PhantomData<Lon>,
    _name: core::marker::PhantomData<Name>,
    _score: core::marker::PhantomData<Score>,
    _tokens: core::marker::PhantomData<Tokens>,
    _type: core::marker::PhantomData<Type>,
}

impl<Areas, Id, Lat, Lon, Name, Score, Tokens, Type>
    MatchBuilder<Areas, Id, Lat, Lon, Name, Score, Tokens, Type>
{
    /// list of areas
    #[inline]
    pub fn areas(
        mut self,
        value: impl Iterator<
            Item = crate::area::AreaBuilder<
                crate::generics::AdminLevelExists,
                crate::generics::MatchedExists,
                crate::generics::NameExists,
            >,
        >,
    ) -> MatchBuilder<crate::generics::AreasExists, Id, Lat, Lon, Name, Score, Tokens, Type> {
        self.body.areas = value.map(|value| value.into()).collect::<Vec<_>>().into();
        unsafe { std::mem::transmute(self) }
    }

    /// house number
    #[inline]
    pub fn house_number(mut self, value: impl Into<String>) -> Self {
        self.body.house_number = Some(value.into());
        self
    }

    /// unique ID of the location
    #[inline]
    pub fn id(
        mut self,
        value: impl Into<String>,
    ) -> MatchBuilder<Areas, crate::generics::IdExists, Lat, Lon, Name, Score, Tokens, Type> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// latitude
    #[inline]
    pub fn lat(
        mut self,
        value: impl Into<f64>,
    ) -> MatchBuilder<Areas, Id, crate::generics::LatExists, Lon, Name, Score, Tokens, Type> {
        self.body.lat = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// level according to OpenStreetMap
    /// (at the moment only for public transport)
    ///
    #[inline]
    pub fn level(mut self, value: impl Into<f64>) -> Self {
        self.body.level = Some(value.into());
        self
    }

    /// longitude
    #[inline]
    pub fn lon(
        mut self,
        value: impl Into<f64>,
    ) -> MatchBuilder<Areas, Id, Lat, crate::generics::LonExists, Name, Score, Tokens, Type> {
        self.body.lon = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// name of the location (transit stop / PoI / address)
    #[inline]
    pub fn name(
        mut self,
        value: impl Into<String>,
    ) -> MatchBuilder<Areas, Id, Lat, Lon, crate::generics::NameExists, Score, Tokens, Type> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// score according to the internal scoring system (the scoring algorithm might change in the future)
    #[inline]
    pub fn score(
        mut self,
        value: impl Into<f64>,
    ) -> MatchBuilder<Areas, Id, Lat, Lon, Name, crate::generics::ScoreExists, Tokens, Type> {
        self.body.score = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// street name
    #[inline]
    pub fn street(mut self, value: impl Into<String>) -> Self {
        self.body.street = Some(value.into());
        self
    }

    /// list of non-overlapping tokens that were matched
    #[inline]
    pub fn tokens(
        mut self,
        value: impl Iterator<Item = impl Iterator<Item = impl Into<f64>>>,
    ) -> MatchBuilder<Areas, Id, Lat, Lon, Name, Score, crate::generics::TokensExists, Type> {
        self.body.tokens = value
            .map(|value| value.map(|value| value.into()).collect::<Vec<_>>().into())
            .collect::<Vec<_>>()
            .into();
        unsafe { std::mem::transmute(self) }
    }

    /// location type
    #[inline]
    pub fn type_(
        mut self,
        value: crate::r#match::MatchType,
    ) -> MatchBuilder<Areas, Id, Lat, Lon, Name, Score, Tokens, crate::generics::TypeExists> {
        self.body.type_ = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// zip code
    #[inline]
    pub fn zip(mut self, value: impl Into<String>) -> Self {
        self.body.zip = Some(value.into());
        self
    }
}
