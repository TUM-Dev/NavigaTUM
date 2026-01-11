#![forbid(unsafe_code)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, ClientInfo, Error, ResponseValue};
use progenitor_client::{ClientHooks, OperationInfo, RequestBuilderExt, encode_path};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    /// Error types.
    pub mod error {
        /// Error from a `TryFrom` or `FromStr` implementation.
        pub struct ConversionError(::std::borrow::Cow<'static, str>);
        impl ::std::error::Error for ConversionError {}
        impl ::std::fmt::Display for ConversionError {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl ::std::fmt::Debug for ConversionError {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }

        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }

    ///An alert, indicating some sort of incident in the public transit
    /// network.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "An alert, indicating some sort of incident in the
    /// public transit network.",
    ///  "type": "object",
    ///  "required": [
    ///    "descriptionText",
    ///    "headerText"
    ///  ],
    ///  "properties": {
    ///    "cause": {
    ///      "$ref": "#/components/schemas/AlertCause"
    ///    },
    ///    "causeDetail": {
    ///      "description": "Description of the cause of the alert that allows
    /// for agency-specific language;\nmore specific than the Cause.\n",
    ///      "type": "string"
    ///    },
    ///    "communicationPeriod": {
    ///      "description": "Time when the alert should be shown to the
    /// user.\nIf missing, the alert will be shown as long as it appears in the
    /// feed.\nIf multiple ranges are given, the alert will be shown during all
    /// of them.\n",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/TimeRange"
    ///      }
    ///    },
    ///    "descriptionText": {
    ///      "description": "Description for the alert.\nThis plain-text string
    /// will be formatted as the body of the alert (or shown on an explicit
    /// \"expand\" request by the user).\nThe information in the description
    /// should add to the information of the header.\n",
    ///      "type": "string"
    ///    },
    ///    "effect": {
    ///      "$ref": "#/components/schemas/AlertEffect"
    ///    },
    ///    "effectDetail": {
    ///      "description": "Description of the effect of the alert that allows
    /// for agency-specific language;\nmore specific than the Effect.\n",
    ///      "type": "string"
    ///    },
    ///    "headerText": {
    ///      "description": "Header for the alert. This plain-text string will
    /// be highlighted, for example in boldface.\n",
    ///      "type": "string"
    ///    },
    ///    "imageAlternativeText": {
    ///      "description": "Text describing the appearance of the linked image
    /// in the image field\n(e.g., in case the image can't be displayed or the
    /// user can't see the image for accessibility reasons).\nSee the HTML spec
    /// for alt image text.\n",
    ///      "type": "string"
    ///    },
    ///    "imageMediaType": {
    ///      "description": "IANA media type as to specify the type of image to
    /// be displayed. The type must start with \"image/\"\n",
    ///      "type": "string"
    ///    },
    ///    "imageUrl": {
    ///      "description": "String containing an URL linking to an image.",
    ///      "type": "string"
    ///    },
    ///    "impactPeriod": {
    ///      "description": "Time when the services are affected by the
    /// disruption mentioned in the alert.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/TimeRange"
    ///      }
    ///    },
    ///    "severityLevel": {
    ///      "$ref": "#/components/schemas/AlertSeverityLevel"
    ///    },
    ///    "ttsDescriptionText": {
    ///      "description": "Text containing a description for the alert to be
    /// used for text-to-speech implementations.\nThis field is the
    /// text-to-speech version of description_text.\nIt should contain the same
    /// information as description_text but formatted such that it can be read
    /// as text-to-speech\n(for example, abbreviations removed, numbers spelled
    /// out, etc.)\n",
    ///      "type": "string"
    ///    },
    ///    "ttsHeaderText": {
    ///      "description": "Text containing the alert's header to be used for
    /// text-to-speech implementations.\nThis field is the text-to-speech
    /// version of header_text.\nIt should contain the same information as
    /// headerText but formatted such that it can read as text-to-speech\n(for
    /// example, abbreviations removed, numbers spelled out, etc.)\n",
    ///      "type": "string"
    ///    },
    ///    "url": {
    ///      "description": "The URL which provides additional information about
    /// the alert.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Alert {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub cause: ::std::option::Option<AlertCause>,
        ///Description of the cause of the alert that allows for
        /// agency-specific language; more specific than the Cause.
        #[serde(
            rename = "causeDetail",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub cause_detail: ::std::option::Option<::std::string::String>,
        ///Time when the alert should be shown to the user.
        ///If missing, the alert will be shown as long as it appears in the
        /// feed. If multiple ranges are given, the alert will be shown
        /// during all of them.
        #[serde(
            rename = "communicationPeriod",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        pub communication_period: ::std::vec::Vec<TimeRange>,
        ///Description for the alert.
        ///This plain-text string will be formatted as the body of the alert
        /// (or shown on an explicit "expand" request by the user).
        /// The information in the description should add to the information of
        /// the header.
        #[serde(rename = "descriptionText")]
        pub description_text: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub effect: ::std::option::Option<AlertEffect>,
        ///Description of the effect of the alert that allows for
        /// agency-specific language; more specific than the Effect.
        #[serde(
            rename = "effectDetail",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub effect_detail: ::std::option::Option<::std::string::String>,
        ///Header for the alert. This plain-text string will be highlighted,
        /// for example in boldface.
        #[serde(rename = "headerText")]
        pub header_text: ::std::string::String,
        ///Text describing the appearance of the linked image in the image
        /// field (e.g., in case the image can't be displayed or the
        /// user can't see the image for accessibility reasons). See the
        /// HTML spec for alt image text.
        #[serde(
            rename = "imageAlternativeText",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub image_alternative_text: ::std::option::Option<::std::string::String>,
        ///IANA media type as to specify the type of image to be displayed. The
        /// type must start with "image/"
        #[serde(
            rename = "imageMediaType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub image_media_type: ::std::option::Option<::std::string::String>,
        ///String containing an URL linking to an image.
        #[serde(
            rename = "imageUrl",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub image_url: ::std::option::Option<::std::string::String>,
        ///Time when the services are affected by the disruption mentioned in
        /// the alert.
        #[serde(
            rename = "impactPeriod",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        pub impact_period: ::std::vec::Vec<TimeRange>,
        #[serde(
            rename = "severityLevel",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub severity_level: ::std::option::Option<AlertSeverityLevel>,
        ///Text containing a description for the alert to be used for
        /// text-to-speech implementations. This field is the
        /// text-to-speech version of description_text.
        /// It should contain the same information as description_text but
        /// formatted such that it can be read as text-to-speech
        /// (for example, abbreviations removed, numbers spelled out, etc.)
        #[serde(
            rename = "ttsDescriptionText",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub tts_description_text: ::std::option::Option<::std::string::String>,
        ///Text containing the alert's header to be used for text-to-speech
        /// implementations. This field is the text-to-speech version of
        /// header_text. It should contain the same information as
        /// headerText but formatted such that it can read as text-to-speech
        /// (for example, abbreviations removed, numbers spelled out, etc.)
        #[serde(
            rename = "ttsHeaderText",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub tts_header_text: ::std::option::Option<::std::string::String>,
        ///The URL which provides additional information about the alert.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub url: ::std::option::Option<::std::string::String>,
    }

    impl ::std::convert::From<&Alert> for Alert {
        fn from(value: &Alert) -> Self {
            value.clone()
        }
    }

    impl Alert {
        pub fn builder() -> builder::Alert {
            Default::default()
        }
    }

    ///Cause of this alert.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Cause of this alert.",
    ///  "type": "string",
    ///  "enum": [
    ///    "UNKNOWN_CAUSE",
    ///    "OTHER_CAUSE",
    ///    "TECHNICAL_PROBLEM",
    ///    "STRIKE",
    ///    "DEMONSTRATION",
    ///    "ACCIDENT",
    ///    "HOLIDAY",
    ///    "WEATHER",
    ///    "MAINTENANCE",
    ///    "CONSTRUCTION",
    ///    "POLICE_ACTIVITY",
    ///    "MEDICAL_EMERGENCY"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum AlertCause {
        #[serde(rename = "UNKNOWN_CAUSE")]
        UnknownCause,
        #[serde(rename = "OTHER_CAUSE")]
        OtherCause,
        #[serde(rename = "TECHNICAL_PROBLEM")]
        TechnicalProblem,
        #[serde(rename = "STRIKE")]
        Strike,
        #[serde(rename = "DEMONSTRATION")]
        Demonstration,
        #[serde(rename = "ACCIDENT")]
        Accident,
        #[serde(rename = "HOLIDAY")]
        Holiday,
        #[serde(rename = "WEATHER")]
        Weather,
        #[serde(rename = "MAINTENANCE")]
        Maintenance,
        #[serde(rename = "CONSTRUCTION")]
        Construction,
        #[serde(rename = "POLICE_ACTIVITY")]
        PoliceActivity,
        #[serde(rename = "MEDICAL_EMERGENCY")]
        MedicalEmergency,
    }

    impl ::std::convert::From<&Self> for AlertCause {
        fn from(value: &AlertCause) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for AlertCause {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::UnknownCause => f.write_str("UNKNOWN_CAUSE"),
                Self::OtherCause => f.write_str("OTHER_CAUSE"),
                Self::TechnicalProblem => f.write_str("TECHNICAL_PROBLEM"),
                Self::Strike => f.write_str("STRIKE"),
                Self::Demonstration => f.write_str("DEMONSTRATION"),
                Self::Accident => f.write_str("ACCIDENT"),
                Self::Holiday => f.write_str("HOLIDAY"),
                Self::Weather => f.write_str("WEATHER"),
                Self::Maintenance => f.write_str("MAINTENANCE"),
                Self::Construction => f.write_str("CONSTRUCTION"),
                Self::PoliceActivity => f.write_str("POLICE_ACTIVITY"),
                Self::MedicalEmergency => f.write_str("MEDICAL_EMERGENCY"),
            }
        }
    }

    impl ::std::str::FromStr for AlertCause {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "UNKNOWN_CAUSE" => Ok(Self::UnknownCause),
                "OTHER_CAUSE" => Ok(Self::OtherCause),
                "TECHNICAL_PROBLEM" => Ok(Self::TechnicalProblem),
                "STRIKE" => Ok(Self::Strike),
                "DEMONSTRATION" => Ok(Self::Demonstration),
                "ACCIDENT" => Ok(Self::Accident),
                "HOLIDAY" => Ok(Self::Holiday),
                "WEATHER" => Ok(Self::Weather),
                "MAINTENANCE" => Ok(Self::Maintenance),
                "CONSTRUCTION" => Ok(Self::Construction),
                "POLICE_ACTIVITY" => Ok(Self::PoliceActivity),
                "MEDICAL_EMERGENCY" => Ok(Self::MedicalEmergency),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for AlertCause {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for AlertCause {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for AlertCause {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///The effect of this problem on the affected entity.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The effect of this problem on the affected entity.",
    ///  "type": "string",
    ///  "enum": [
    ///    "NO_SERVICE",
    ///    "REDUCED_SERVICE",
    ///    "SIGNIFICANT_DELAYS",
    ///    "DETOUR",
    ///    "ADDITIONAL_SERVICE",
    ///    "MODIFIED_SERVICE",
    ///    "OTHER_EFFECT",
    ///    "UNKNOWN_EFFECT",
    ///    "STOP_MOVED",
    ///    "NO_EFFECT",
    ///    "ACCESSIBILITY_ISSUE"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum AlertEffect {
        #[serde(rename = "NO_SERVICE")]
        NoService,
        #[serde(rename = "REDUCED_SERVICE")]
        ReducedService,
        #[serde(rename = "SIGNIFICANT_DELAYS")]
        SignificantDelays,
        #[serde(rename = "DETOUR")]
        Detour,
        #[serde(rename = "ADDITIONAL_SERVICE")]
        AdditionalService,
        #[serde(rename = "MODIFIED_SERVICE")]
        ModifiedService,
        #[serde(rename = "OTHER_EFFECT")]
        OtherEffect,
        #[serde(rename = "UNKNOWN_EFFECT")]
        UnknownEffect,
        #[serde(rename = "STOP_MOVED")]
        StopMoved,
        #[serde(rename = "NO_EFFECT")]
        NoEffect,
        #[serde(rename = "ACCESSIBILITY_ISSUE")]
        AccessibilityIssue,
    }

    impl ::std::convert::From<&Self> for AlertEffect {
        fn from(value: &AlertEffect) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for AlertEffect {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::NoService => f.write_str("NO_SERVICE"),
                Self::ReducedService => f.write_str("REDUCED_SERVICE"),
                Self::SignificantDelays => f.write_str("SIGNIFICANT_DELAYS"),
                Self::Detour => f.write_str("DETOUR"),
                Self::AdditionalService => f.write_str("ADDITIONAL_SERVICE"),
                Self::ModifiedService => f.write_str("MODIFIED_SERVICE"),
                Self::OtherEffect => f.write_str("OTHER_EFFECT"),
                Self::UnknownEffect => f.write_str("UNKNOWN_EFFECT"),
                Self::StopMoved => f.write_str("STOP_MOVED"),
                Self::NoEffect => f.write_str("NO_EFFECT"),
                Self::AccessibilityIssue => f.write_str("ACCESSIBILITY_ISSUE"),
            }
        }
    }

    impl ::std::str::FromStr for AlertEffect {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "NO_SERVICE" => Ok(Self::NoService),
                "REDUCED_SERVICE" => Ok(Self::ReducedService),
                "SIGNIFICANT_DELAYS" => Ok(Self::SignificantDelays),
                "DETOUR" => Ok(Self::Detour),
                "ADDITIONAL_SERVICE" => Ok(Self::AdditionalService),
                "MODIFIED_SERVICE" => Ok(Self::ModifiedService),
                "OTHER_EFFECT" => Ok(Self::OtherEffect),
                "UNKNOWN_EFFECT" => Ok(Self::UnknownEffect),
                "STOP_MOVED" => Ok(Self::StopMoved),
                "NO_EFFECT" => Ok(Self::NoEffect),
                "ACCESSIBILITY_ISSUE" => Ok(Self::AccessibilityIssue),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for AlertEffect {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for AlertEffect {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for AlertEffect {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///The severity of the alert.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The severity of the alert.",
    ///  "type": "string",
    ///  "enum": [
    ///    "UNKNOWN_SEVERITY",
    ///    "INFO",
    ///    "WARNING",
    ///    "SEVERE"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum AlertSeverityLevel {
        #[serde(rename = "UNKNOWN_SEVERITY")]
        UnknownSeverity,
        #[serde(rename = "INFO")]
        Info,
        #[serde(rename = "WARNING")]
        Warning,
        #[serde(rename = "SEVERE")]
        Severe,
    }

    impl ::std::convert::From<&Self> for AlertSeverityLevel {
        fn from(value: &AlertSeverityLevel) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for AlertSeverityLevel {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::UnknownSeverity => f.write_str("UNKNOWN_SEVERITY"),
                Self::Info => f.write_str("INFO"),
                Self::Warning => f.write_str("WARNING"),
                Self::Severe => f.write_str("SEVERE"),
            }
        }
    }

    impl ::std::str::FromStr for AlertSeverityLevel {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "UNKNOWN_SEVERITY" => Ok(Self::UnknownSeverity),
                "INFO" => Ok(Self::Info),
                "WARNING" => Ok(Self::Warning),
                "SEVERE" => Ok(Self::Severe),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for AlertSeverityLevel {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for AlertSeverityLevel {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for AlertSeverityLevel {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///Administrative area
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Administrative area",
    ///  "type": "object",
    ///  "required": [
    ///    "adminLevel",
    ///    "matched",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "adminLevel": {
    ///      "description": "[OpenStreetMap `admin_level`](https://wiki.openstreetmap.org/wiki/Key:admin_level)\nof the area\n",
    ///      "type": "number"
    ///    },
    ///    "default": {
    ///      "description": "Whether this area should be displayed as default
    /// area (area with admin level closest 7)",
    ///      "type": "boolean"
    ///    },
    ///    "matched": {
    ///      "description": "Whether this area was matched by the input text",
    ///      "type": "boolean"
    ///    },
    ///    "name": {
    ///      "description": "Name of the area",
    ///      "type": "string"
    ///    },
    ///    "unique": {
    ///      "description": "Set for the first area after the `default` area
    /// that distinguishes areas\nif the match is ambiguous regarding (`default`
    /// area + place name / street [+ house number]).\n",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Area {
        #[serde(rename = "adminLevel")]
        pub admin_level: f64,
        ///Whether this area should be displayed as default area (area with
        /// admin level closest 7)
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub default: ::std::option::Option<bool>,
        ///Whether this area was matched by the input text
        pub matched: bool,
        ///Name of the area
        pub name: ::std::string::String,
        ///Set for the first area after the `default` area that distinguishes
        /// areas if the match is ambiguous regarding (`default` area +
        /// place name / street [+ house number]).
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub unique: ::std::option::Option<bool>,
    }

    impl ::std::convert::From<&Area> for Area {
        fn from(value: &Area) -> Self {
            value.clone()
        }
    }

    impl Area {
        pub fn builder() -> builder::Area {
            Default::default()
        }
    }

    ///`Direction`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "DEPART",
    ///    "HARD_LEFT",
    ///    "LEFT",
    ///    "SLIGHTLY_LEFT",
    ///    "CONTINUE",
    ///    "SLIGHTLY_RIGHT",
    ///    "RIGHT",
    ///    "HARD_RIGHT",
    ///    "CIRCLE_CLOCKWISE",
    ///    "CIRCLE_COUNTERCLOCKWISE",
    ///    "STAIRS",
    ///    "ELEVATOR",
    ///    "UTURN_LEFT",
    ///    "UTURN_RIGHT"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum Direction {
        #[serde(rename = "DEPART")]
        Depart,
        #[serde(rename = "HARD_LEFT")]
        HardLeft,
        #[serde(rename = "LEFT")]
        Left,
        #[serde(rename = "SLIGHTLY_LEFT")]
        SlightlyLeft,
        #[serde(rename = "CONTINUE")]
        Continue,
        #[serde(rename = "SLIGHTLY_RIGHT")]
        SlightlyRight,
        #[serde(rename = "RIGHT")]
        Right,
        #[serde(rename = "HARD_RIGHT")]
        HardRight,
        #[serde(rename = "CIRCLE_CLOCKWISE")]
        CircleClockwise,
        #[serde(rename = "CIRCLE_COUNTERCLOCKWISE")]
        CircleCounterclockwise,
        #[serde(rename = "STAIRS")]
        Stairs,
        #[serde(rename = "ELEVATOR")]
        Elevator,
        #[serde(rename = "UTURN_LEFT")]
        UturnLeft,
        #[serde(rename = "UTURN_RIGHT")]
        UturnRight,
    }

    impl ::std::convert::From<&Self> for Direction {
        fn from(value: &Direction) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for Direction {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Depart => f.write_str("DEPART"),
                Self::HardLeft => f.write_str("HARD_LEFT"),
                Self::Left => f.write_str("LEFT"),
                Self::SlightlyLeft => f.write_str("SLIGHTLY_LEFT"),
                Self::Continue => f.write_str("CONTINUE"),
                Self::SlightlyRight => f.write_str("SLIGHTLY_RIGHT"),
                Self::Right => f.write_str("RIGHT"),
                Self::HardRight => f.write_str("HARD_RIGHT"),
                Self::CircleClockwise => f.write_str("CIRCLE_CLOCKWISE"),
                Self::CircleCounterclockwise => f.write_str("CIRCLE_COUNTERCLOCKWISE"),
                Self::Stairs => f.write_str("STAIRS"),
                Self::Elevator => f.write_str("ELEVATOR"),
                Self::UturnLeft => f.write_str("UTURN_LEFT"),
                Self::UturnRight => f.write_str("UTURN_RIGHT"),
            }
        }
    }

    impl ::std::str::FromStr for Direction {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "DEPART" => Ok(Self::Depart),
                "HARD_LEFT" => Ok(Self::HardLeft),
                "LEFT" => Ok(Self::Left),
                "SLIGHTLY_LEFT" => Ok(Self::SlightlyLeft),
                "CONTINUE" => Ok(Self::Continue),
                "SLIGHTLY_RIGHT" => Ok(Self::SlightlyRight),
                "RIGHT" => Ok(Self::Right),
                "HARD_RIGHT" => Ok(Self::HardRight),
                "CIRCLE_CLOCKWISE" => Ok(Self::CircleClockwise),
                "CIRCLE_COUNTERCLOCKWISE" => Ok(Self::CircleCounterclockwise),
                "STAIRS" => Ok(Self::Stairs),
                "ELEVATOR" => Ok(Self::Elevator),
                "UTURN_LEFT" => Ok(Self::UturnLeft),
                "UTURN_RIGHT" => Ok(Self::UturnRight),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for Direction {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for Direction {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for Direction {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///Object containing duration if a path was found or none if no path was
    /// found
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Object containing duration if a path was found or none
    /// if no path was found",
    ///  "type": "object",
    ///  "properties": {
    ///    "duration": {
    ///      "description": "duration in seconds if a path was found, otherwise
    /// missing",
    ///      "type": "number"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Duration {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub duration: ::std::option::Option<f64>,
    }

    impl ::std::convert::From<&Duration> for Duration {
        fn from(value: &Duration) -> Self {
            value.clone()
        }
    }

    impl ::std::default::Default for Duration {
        fn default() -> Self {
            Self {
                duration: Default::default(),
            }
        }
    }

    impl Duration {
        pub fn builder() -> builder::Duration {
            Default::default()
        }
    }

    ///Different elevation cost profiles for street routing.
    ///Using a elevation cost profile will prefer routes with a smaller incline
    /// and smaller difference in elevation, even if the routed way is longer.
    ///
    /// - `NONE`: Ignore elevation data for routing. This is the default
    ///   behavior
    /// - `LOW`: Add a low penalty for inclines. This will favor longer paths,
    ///   if the elevation increase and incline are smaller.
    /// - `HIGH`: Add a high penalty for inclines. This will favor even longer
    ///   paths, if the elevation increase and incline are smaller.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Different elevation cost profiles for street
    /// routing.\nUsing a elevation cost profile will prefer routes with a
    /// smaller incline and smaller difference in elevation, even if the routed
    /// way is longer.\n\n- `NONE`: Ignore elevation data for routing. This is
    /// the default behavior\n- `LOW`: Add a low penalty for inclines. This will
    /// favor longer paths, if the elevation increase and incline are
    /// smaller.\n- `HIGH`: Add a high penalty for inclines. This will favor
    /// even longer paths, if the elevation increase and incline are
    /// smaller.\n",
    ///  "type": "string",
    ///  "enum": [
    ///    "NONE",
    ///    "LOW",
    ///    "HIGH"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum ElevationCosts {
        #[serde(rename = "NONE")]
        None,
        #[serde(rename = "LOW")]
        Low,
        #[serde(rename = "HIGH")]
        High,
    }

    impl ::std::convert::From<&Self> for ElevationCosts {
        fn from(value: &ElevationCosts) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for ElevationCosts {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::None => f.write_str("NONE"),
                Self::Low => f.write_str("LOW"),
                Self::High => f.write_str("HIGH"),
            }
        }
    }

    impl ::std::str::FromStr for ElevationCosts {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "NONE" => Ok(Self::None),
                "LOW" => Ok(Self::Low),
                "HIGH" => Ok(Self::High),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for ElevationCosts {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for ElevationCosts {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for ElevationCosts {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`EncodedPolyline`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "length",
    ///    "points",
    ///    "precision"
    ///  ],
    ///  "properties": {
    ///    "length": {
    ///      "description": "The number of points in the string",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "points": {
    ///      "description": "The encoded points of the polyline using the Google
    /// polyline encoding.",
    ///      "type": "string"
    ///    },
    ///    "precision": {
    ///      "description": "The precision of the returned polyline (7 for /v1,
    /// 6 for /v2)\nBe aware that with precision 7, coordinates with |longitude|
    /// > 107.37 are undefined/will overflow.\n",
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct EncodedPolyline {
        ///The number of points in the string
        pub length: u64,
        ///The encoded points of the polyline using the Google polyline
        /// encoding.
        pub points: ::std::string::String,
        ///The precision of the returned polyline (7 for /v1, 6 for /v2)
        ///Be aware that with precision 7, coordinates with |longitude| >
        /// 107.37 are undefined/will overflow.
        pub precision: i64,
    }

    impl ::std::convert::From<&EncodedPolyline> for EncodedPolyline {
        fn from(value: &EncodedPolyline) -> Self {
            value.clone()
        }
    }

    impl EncodedPolyline {
        pub fn builder() -> builder::EncodedPolyline {
            Default::default()
        }
    }

    ///`FareMedia`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "fareMediaType"
    ///  ],
    ///  "properties": {
    ///    "fareMediaName": {
    ///      "description": "Name of the fare media. Required for transit cards
    /// and mobile apps.",
    ///      "type": "string"
    ///    },
    ///    "fareMediaType": {
    ///      "$ref": "#/components/schemas/FareMediaType"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct FareMedia {
        ///Name of the fare media. Required for transit cards and mobile apps.
        #[serde(
            rename = "fareMediaName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub fare_media_name: ::std::option::Option<::std::string::String>,
        #[serde(rename = "fareMediaType")]
        pub fare_media_type: FareMediaType,
    }

    impl ::std::convert::From<&FareMedia> for FareMedia {
        fn from(value: &FareMedia) -> Self {
            value.clone()
        }
    }

    impl FareMedia {
        pub fn builder() -> builder::FareMedia {
            Default::default()
        }
    }

    /// - `NONE`: No fare media involved (e.g., cash payment)
    /// - `PAPER_TICKET`: Physical paper ticket
    /// - `TRANSIT_CARD`: Physical transit card with stored value
    /// - `CONTACTLESS_EMV`: cEMV (contactless payment)
    /// - `MOBILE_APP`: Mobile app with virtual transit cards/passes
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "- `NONE`: No fare media involved (e.g., cash
    /// payment)\n- `PAPER_TICKET`: Physical paper ticket\n- `TRANSIT_CARD`:
    /// Physical transit card with stored value\n- `CONTACTLESS_EMV`: cEMV
    /// (contactless payment)\n- `MOBILE_APP`: Mobile app with virtual transit
    /// cards/passes\n",
    ///  "type": "string",
    ///  "enum": [
    ///    "NONE",
    ///    "PAPER_TICKET",
    ///    "TRANSIT_CARD",
    ///    "CONTACTLESS_EMV",
    ///    "MOBILE_APP"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum FareMediaType {
        #[serde(rename = "NONE")]
        None,
        #[serde(rename = "PAPER_TICKET")]
        PaperTicket,
        #[serde(rename = "TRANSIT_CARD")]
        TransitCard,
        #[serde(rename = "CONTACTLESS_EMV")]
        ContactlessEmv,
        #[serde(rename = "MOBILE_APP")]
        MobileApp,
    }

    impl ::std::convert::From<&Self> for FareMediaType {
        fn from(value: &FareMediaType) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for FareMediaType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::None => f.write_str("NONE"),
                Self::PaperTicket => f.write_str("PAPER_TICKET"),
                Self::TransitCard => f.write_str("TRANSIT_CARD"),
                Self::ContactlessEmv => f.write_str("CONTACTLESS_EMV"),
                Self::MobileApp => f.write_str("MOBILE_APP"),
            }
        }
    }

    impl ::std::str::FromStr for FareMediaType {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "NONE" => Ok(Self::None),
                "PAPER_TICKET" => Ok(Self::PaperTicket),
                "TRANSIT_CARD" => Ok(Self::TransitCard),
                "CONTACTLESS_EMV" => Ok(Self::ContactlessEmv),
                "MOBILE_APP" => Ok(Self::MobileApp),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for FareMediaType {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for FareMediaType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for FareMediaType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`FareProduct`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "amount",
    ///    "currency",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "amount": {
    ///      "description": "The cost of the fare product. May be negative to
    /// represent transfer discounts. May be zero to represent a fare product
    /// that is free.",
    ///      "type": "number"
    ///    },
    ///    "currency": {
    ///      "description": "ISO 4217 currency code. The currency of the cost of
    /// the fare product.",
    ///      "type": "string"
    ///    },
    ///    "media": {
    ///      "$ref": "#/components/schemas/FareMedia"
    ///    },
    ///    "name": {
    ///      "description": "The name of the fare product as displayed to
    /// riders.",
    ///      "type": "string"
    ///    },
    ///    "riderCategory": {
    ///      "$ref": "#/components/schemas/RiderCategory"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct FareProduct {
        pub amount: f64,
        ///ISO 4217 currency code. The currency of the cost of the fare
        /// product.
        pub currency: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub media: ::std::option::Option<FareMedia>,
        ///The name of the fare product as displayed to riders.
        pub name: ::std::string::String,
        #[serde(
            rename = "riderCategory",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub rider_category: ::std::option::Option<RiderCategory>,
    }

    impl ::std::convert::From<&FareProduct> for FareProduct {
        fn from(value: &FareProduct) -> Self {
            value.clone()
        }
    }

    impl FareProduct {
        pub fn builder() -> builder::FareProduct {
            Default::default()
        }
    }

    ///The concept is derived from: https://gtfs.org/documentation/schedule/reference/#fare_transfer_rulestxt
    ///
    ///Terminology:
    ///  - **Leg**: An itinerary leg as described by the `Leg` type of this API
    ///    description.
    ///  - **Effective Fare Leg**: Itinerary legs can be joined together to form
    ///    one *effective fare leg*.
    ///  - **Fare Transfer**: A fare transfer groups two or more effective fare
    ///    legs.
    ///  - **A** is the first *effective fare leg* of potentially multiple
    ///    consecutive legs contained in a fare transfer
    ///  - **B** is any *effective fare leg* following the first *effective fare
    ///    leg* in this transfer
    ///  - **AB** are all changes between *effective fare legs* contained in
    ///    this transfer
    ///
    ///The fare transfer rule is used to derive the final set of products of
    /// the itinerary legs contained in this transfer:
    ///  - A_AB means that any product from the first effective fare leg
    ///    combined with the product attached to the transfer itself (AB) which
    ///    can be empty (= free). Note that all subsequent effective fare leg
    ///    products need to be ignored in this case.
    ///  - A_AB_B mean that a product for each effective fare leg needs to be
    ///    purchased in a addition to the product attached to the transfer
    ///    itself (AB) which can be empty (= free)
    ///  - AB only the transfer product itself has to be purchased. Note that
    ///    all fare products attached to the contained effective fare legs need
    ///    to be ignored in this case.
    ///
    ///An itinerary `Leg` references the index of the fare transfer and the
    /// index of the effective fare leg in this transfer it belongs to.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The concept is derived from: https://gtfs.org/documentation/schedule/reference/#fare_transfer_rulestxt\n\nTerminology:\n  - **Leg**: An itinerary leg as described by the `Leg` type of this API description.\n  - **Effective Fare Leg**: Itinerary legs can be joined together to form one *effective fare leg*.\n  - **Fare Transfer**: A fare transfer groups two or more effective fare legs.\n  - **A** is the first *effective fare leg* of potentially multiple consecutive legs contained in a fare transfer\n  - **B** is any *effective fare leg* following the first *effective fare leg* in this transfer\n  - **AB** are all changes between *effective fare legs* contained in this transfer\n\nThe fare transfer rule is used to derive the final set of products of the itinerary legs contained in this transfer:\n  - A_AB means that any product from the first effective fare leg combined with the product attached to the transfer itself (AB) which can be empty (= free). Note that all subsequent effective fare leg products need to be ignored in this case.\n  - A_AB_B mean that a product for each effective fare leg needs to be purchased in a addition to the product attached to the transfer itself (AB) which can be empty (= free)\n  - AB only the transfer product itself has to be purchased. Note that all fare products attached to the contained effective fare legs need to be ignored in this case.\n\nAn itinerary `Leg` references the index of the fare transfer and the index of the effective fare leg in this transfer it belongs to.\n",
    ///  "type": "object",
    ///  "required": [
    ///    "effectiveFareLegProducts"
    ///  ],
    ///  "properties": {
    ///    "effectiveFareLegProducts": {
    ///      "description": "Lists all valid fare products for the effective
    /// fare legs.\nThis is an `array<array<FareProduct>>` where the inner
    /// array\nlists all possible fare products that would cover this effective
    /// fare leg.\nEach \"effective fare leg\" can have multiple options for
    /// adult/child/weekly/monthly/day/one-way tickets etc.\nYou can see the
    /// outer array as AND (you need one ticket for each effective fare leg
    /// (`A_AB_B`), the first effective fare leg (`A_AB`) or no fare leg at all
    /// but only the transfer product (`AB`)\nand the inner array as OR (you can
    /// choose which ticket to buy)\n",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "array",
    ///        "items": {
    ///          "type": "array",
    ///          "items": {
    ///            "$ref": "#/components/schemas/FareProduct"
    ///          }
    ///        }
    ///      }
    ///    },
    ///    "rule": {
    ///      "$ref": "#/components/schemas/FareTransferRule"
    ///    },
    ///    "transferProducts": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/FareProduct"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct FareTransfer {
        ///Lists all valid fare products for the effective fare legs.
        ///This is an `array<array<FareProduct>>` where the inner array
        ///lists all possible fare products that would cover this effective
        /// fare leg. Each "effective fare leg" can have multiple
        /// options for adult/child/weekly/monthly/day/one-way tickets etc.
        /// You can see the outer array as AND (you need one ticket for each
        /// effective fare leg (`A_AB_B`), the first effective fare leg (`A_AB`)
        /// or no fare leg at all but only the transfer product (`AB`)
        /// and the inner array as OR (you can choose which ticket to buy)
        #[serde(rename = "effectiveFareLegProducts")]
        pub effective_fare_leg_products:
            ::std::vec::Vec<::std::vec::Vec<::std::vec::Vec<FareProduct>>>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub rule: ::std::option::Option<FareTransferRule>,
        #[serde(
            rename = "transferProducts",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        pub transfer_products: ::std::vec::Vec<FareProduct>,
    }

    impl ::std::convert::From<&FareTransfer> for FareTransfer {
        fn from(value: &FareTransfer) -> Self {
            value.clone()
        }
    }

    impl FareTransfer {
        pub fn builder() -> builder::FareTransfer {
            Default::default()
        }
    }

    ///`FareTransferRule`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "A_AB",
    ///    "A_AB_B",
    ///    "AB"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum FareTransferRule {
        #[serde(rename = "A_AB")]
        AAb,
        #[serde(rename = "A_AB_B")]
        AAbB,
        #[serde(rename = "AB")]
        Ab,
    }

    impl ::std::convert::From<&Self> for FareTransferRule {
        fn from(value: &FareTransferRule) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for FareTransferRule {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::AAb => f.write_str("A_AB"),
                Self::AAbB => f.write_str("A_AB_B"),
                Self::Ab => f.write_str("AB"),
            }
        }
    }

    impl ::std::str::FromStr for FareTransferRule {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "A_AB" => Ok(Self::AAb),
                "A_AB_B" => Ok(Self::AAbB),
                "AB" => Ok(Self::Ab),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for FareTransferRule {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for FareTransferRule {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for FareTransferRule {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`InitialResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "lat",
    ///    "lon",
    ///    "zoom"
    ///  ],
    ///  "properties": {
    ///    "lat": {
    ///      "description": "latitude",
    ///      "type": "number"
    ///    },
    ///    "lon": {
    ///      "description": "longitude",
    ///      "type": "number"
    ///    },
    ///    "zoom": {
    ///      "description": "zoom level",
    ///      "type": "number"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct InitialResponse {
        pub lat: f64,
        pub lon: f64,
        pub zoom: f64,
    }

    impl ::std::convert::From<&InitialResponse> for InitialResponse {
        fn from(value: &InitialResponse) -> Self {
            value.clone()
        }
    }

    impl InitialResponse {
        pub fn builder() -> builder::InitialResponse {
            Default::default()
        }
    }

    ///`Itinerary`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "duration",
    ///    "endTime",
    ///    "legs",
    ///    "startTime",
    ///    "transfers"
    ///  ],
    ///  "properties": {
    ///    "duration": {
    ///      "description": "journey duration in seconds",
    ///      "type": "integer"
    ///    },
    ///    "endTime": {
    ///      "description": "journey arrival time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "fareTransfers": {
    ///      "description": "Fare information",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/FareTransfer"
    ///      }
    ///    },
    ///    "legs": {
    ///      "description": "Journey legs",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Leg"
    ///      }
    ///    },
    ///    "startTime": {
    ///      "description": "journey departure time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "transfers": {
    ///      "description": "The number of transfers this trip has.",
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Itinerary {
        ///journey duration in seconds
        pub duration: i64,
        ///journey arrival time
        #[serde(rename = "endTime")]
        pub end_time: ::chrono::DateTime<::chrono::offset::Utc>,
        ///Fare information
        #[serde(
            rename = "fareTransfers",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        pub fare_transfers: ::std::vec::Vec<FareTransfer>,
        ///Journey legs
        pub legs: ::std::vec::Vec<Leg>,
        ///journey departure time
        #[serde(rename = "startTime")]
        pub start_time: ::chrono::DateTime<::chrono::offset::Utc>,
        ///The number of transfers this trip has.
        pub transfers: i64,
    }

    impl ::std::convert::From<&Itinerary> for Itinerary {
        fn from(value: &Itinerary) -> Self {
            value.clone()
        }
    }

    impl Itinerary {
        pub fn builder() -> builder::Itinerary {
            Default::default()
        }
    }

    ///`Leg`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "duration",
    ///    "endTime",
    ///    "from",
    ///    "legGeometry",
    ///    "mode",
    ///    "realTime",
    ///    "scheduled",
    ///    "scheduledEndTime",
    ///    "scheduledStartTime",
    ///    "startTime",
    ///    "to"
    ///  ],
    ///  "properties": {
    ///    "agencyId": {
    ///      "type": "string"
    ///    },
    ///    "agencyName": {
    ///      "type": "string"
    ///    },
    ///    "agencyUrl": {
    ///      "type": "string"
    ///    },
    ///    "alerts": {
    ///      "description": "Alerts for this stop.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Alert"
    ///      }
    ///    },
    ///    "cancelled": {
    ///      "description": "Whether this trip is cancelled",
    ///      "type": "boolean"
    ///    },
    ///    "displayName": {
    ///      "type": "string"
    ///    },
    ///    "distance": {
    ///      "description": "For non-transit legs the distance traveled while
    /// traversing this leg in meters.",
    ///      "type": "number"
    ///    },
    ///    "duration": {
    ///      "description": "Leg duration in seconds\n\nIf leg is footpath:\n
    /// The footpath duration is derived from the default footpath\n  duration
    /// using the query parameters `transferTimeFactor` and\n
    /// `additionalTransferTime` as follows:\n  `leg.duration = defaultDuration
    /// * transferTimeFactor + additionalTransferTime.`\n  In case the
    /// defaultDuration is needed, it can be calculated by\n  `defaultDuration =
    /// (leg.duration - additionalTransferTime) / transferTimeFactor`.\n  Note
    /// that the default values are `transferTimeFactor = 1` and\n
    /// `additionalTransferTime = 0` in case they are not explicitly\n  provided
    /// in the query.\n",
    ///      "type": "integer"
    ///    },
    ///    "effectiveFareLegIndex": {
    ///      "description": "Index into the
    /// `Itinerary.fareTransfers[fareTransferIndex].effectiveFareLegProducts`
    /// array\nto identify which effective fare leg this itinerary leg belongs
    /// to\n",
    ///      "type": "integer"
    ///    },
    ///    "endTime": {
    ///      "description": "leg arrival time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "fareTransferIndex": {
    ///      "description": "Index into `Itinerary.fareTransfers` array\nto
    /// identify which fare transfer this leg belongs to\n",
    ///      "type": "integer"
    ///    },
    ///    "from": {
    ///      "$ref": "#/components/schemas/Place"
    ///    },
    ///    "headsign": {
    ///      "description": "For transit legs, the headsign of the bus or train
    /// being used.\nFor non-transit legs, null\n",
    ///      "type": "string"
    ///    },
    ///    "interlineWithPreviousLeg": {
    ///      "description": "For transit legs, if the rider should stay on the
    /// vehicle as it changes route names.",
    ///      "type": "boolean"
    ///    },
    ///    "intermediateStops": {
    ///      "description": "For transit legs, intermediate stops between the
    /// Place where the leg originates\nand the Place where the leg ends. For
    /// non-transit legs, null.\n",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Place"
    ///      }
    ///    },
    ///    "legGeometry": {
    ///      "$ref": "#/components/schemas/EncodedPolyline"
    ///    },
    ///    "loopedCalendarSince": {
    ///      "description": "If set, this attribute indicates that this trip has
    /// been expanded\nbeyond the feed end date (enabled by config flag
    /// `timetable.dataset.extend_calendar`)\nby looping active weekdays, e.g.
    /// from calendar.txt in GTFS.\n",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "mode": {
    ///      "$ref": "#/components/schemas/Mode"
    ///    },
    ///    "realTime": {
    ///      "description": "Whether there is real-time data about this leg",
    ///      "type": "boolean"
    ///    },
    ///    "rental": {
    ///      "$ref": "#/components/schemas/Rental"
    ///    },
    ///    "routeColor": {
    ///      "type": "string"
    ///    },
    ///    "routeLongName": {
    ///      "type": "string"
    ///    },
    ///    "routeShortName": {
    ///      "type": "string"
    ///    },
    ///    "routeTextColor": {
    ///      "type": "string"
    ///    },
    ///    "routeType": {
    ///      "type": "integer"
    ///    },
    ///    "scheduled": {
    ///      "description": "Whether this leg was originally scheduled to run or
    /// is an additional service.\nScheduled times will equal realtime times in
    /// this case.\n",
    ///      "type": "boolean"
    ///    },
    ///    "scheduledEndTime": {
    ///      "description": "scheduled leg arrival time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "scheduledStartTime": {
    ///      "description": "scheduled leg departure time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "source": {
    ///      "description": "Filename and line number where this trip is from",
    ///      "type": "string"
    ///    },
    ///    "startTime": {
    ///      "description": "leg departure time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "steps": {
    ///      "description": "A series of turn by turn instructions\nused for
    /// walking, biking and driving.\n",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/StepInstruction"
    ///      }
    ///    },
    ///    "to": {
    ///      "$ref": "#/components/schemas/Place"
    ///    },
    ///    "tripId": {
    ///      "type": "string"
    ///    },
    ///    "tripShortName": {
    ///      "type": "string"
    ///    },
    ///    "tripTo": {
    ///      "$ref": "#/components/schemas/Place"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Leg {
        #[serde(
            rename = "agencyId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub agency_id: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "agencyName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub agency_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "agencyUrl",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub agency_url: ::std::option::Option<::std::string::String>,
        ///Alerts for this stop.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        pub alerts: ::std::vec::Vec<Alert>,
        ///Whether this trip is cancelled
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub cancelled: ::std::option::Option<bool>,
        #[serde(
            rename = "displayName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub display_name: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub distance: ::std::option::Option<f64>,
        ///Leg duration in seconds
        ///
        ///If leg is footpath:
        ///  The footpath duration is derived from the default footpath
        ///  duration using the query parameters `transferTimeFactor` and
        ///  `additionalTransferTime` as follows:
        ///  `leg.duration = defaultDuration * transferTimeFactor +
        /// additionalTransferTime.`  In case the defaultDuration is
        /// needed, it can be calculated by  `defaultDuration =
        /// (leg.duration - additionalTransferTime) / transferTimeFactor`.
        ///  Note that the default values are `transferTimeFactor = 1` and
        ///  `additionalTransferTime = 0` in case they are not explicitly
        ///  provided in the query.
        pub duration: i64,
        ///Index into the
        /// `Itinerary.fareTransfers[fareTransferIndex].
        /// effectiveFareLegProducts` array to identify which effective
        /// fare leg this itinerary leg belongs to
        #[serde(
            rename = "effectiveFareLegIndex",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub effective_fare_leg_index: ::std::option::Option<i64>,
        ///leg arrival time
        #[serde(rename = "endTime")]
        pub end_time: ::chrono::DateTime<::chrono::offset::Utc>,
        ///Index into `Itinerary.fareTransfers` array
        ///to identify which fare transfer this leg belongs to
        #[serde(
            rename = "fareTransferIndex",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub fare_transfer_index: ::std::option::Option<i64>,
        pub from: Place,
        ///For transit legs, the headsign of the bus or train being used.
        ///For non-transit legs, null
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub headsign: ::std::option::Option<::std::string::String>,
        ///For transit legs, if the rider should stay on the vehicle as it
        /// changes route names.
        #[serde(
            rename = "interlineWithPreviousLeg",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub interline_with_previous_leg: ::std::option::Option<bool>,
        ///For transit legs, intermediate stops between the Place where the leg
        /// originates and the Place where the leg ends. For non-transit
        /// legs, null.
        #[serde(
            rename = "intermediateStops",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        pub intermediate_stops: ::std::vec::Vec<Place>,
        #[serde(rename = "legGeometry")]
        pub leg_geometry: EncodedPolyline,
        ///If set, this attribute indicates that this trip has been expanded
        ///beyond the feed end date (enabled by config flag
        /// `timetable.dataset.extend_calendar`) by looping active
        /// weekdays, e.g. from calendar.txt in GTFS.
        #[serde(
            rename = "loopedCalendarSince",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub looped_calendar_since: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
        pub mode: Mode,
        ///Whether there is real-time data about this leg
        #[serde(rename = "realTime")]
        pub real_time: bool,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub rental: ::std::option::Option<Rental>,
        #[serde(
            rename = "routeColor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_color: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "routeLongName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_long_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "routeShortName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_short_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "routeTextColor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_text_color: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "routeType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_type: ::std::option::Option<i64>,
        ///Whether this leg was originally scheduled to run or is an additional
        /// service. Scheduled times will equal realtime times in this
        /// case.
        pub scheduled: bool,
        ///scheduled leg arrival time
        #[serde(rename = "scheduledEndTime")]
        pub scheduled_end_time: ::chrono::DateTime<::chrono::offset::Utc>,
        ///scheduled leg departure time
        #[serde(rename = "scheduledStartTime")]
        pub scheduled_start_time: ::chrono::DateTime<::chrono::offset::Utc>,
        ///Filename and line number where this trip is from
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub source: ::std::option::Option<::std::string::String>,
        ///leg departure time
        #[serde(rename = "startTime")]
        pub start_time: ::chrono::DateTime<::chrono::offset::Utc>,
        ///A series of turn by turn instructions
        ///used for walking, biking and driving.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        pub steps: ::std::vec::Vec<StepInstruction>,
        pub to: Place,
        #[serde(
            rename = "tripId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub trip_id: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "tripShortName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub trip_short_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "tripTo",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub trip_to: ::std::option::Option<Place>,
    }

    impl ::std::convert::From<&Leg> for Leg {
        fn from(value: &Leg) -> Self {
            value.clone()
        }
    }

    impl Leg {
        pub fn builder() -> builder::Leg {
            Default::default()
        }
    }

    ///location type
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "location type",
    ///  "type": "string",
    ///  "enum": [
    ///    "ADDRESS",
    ///    "PLACE",
    ///    "STOP"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum LocationType {
        #[serde(rename = "ADDRESS")]
        Address,
        #[serde(rename = "PLACE")]
        Place,
        #[serde(rename = "STOP")]
        Stop,
    }

    impl ::std::convert::From<&Self> for LocationType {
        fn from(value: &LocationType) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for LocationType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Address => f.write_str("ADDRESS"),
                Self::Place => f.write_str("PLACE"),
                Self::Stop => f.write_str("STOP"),
            }
        }
    }

    impl ::std::str::FromStr for LocationType {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "ADDRESS" => Ok(Self::Address),
                "PLACE" => Ok(Self::Place),
                "STOP" => Ok(Self::Stop),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for LocationType {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for LocationType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for LocationType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///GeoCoding match
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "GeoCoding match",
    ///  "type": "object",
    ///  "required": [
    ///    "areas",
    ///    "id",
    ///    "lat",
    ///    "lon",
    ///    "name",
    ///    "score",
    ///    "tokens",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "areas": {
    ///      "description": "list of areas",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Area"
    ///      }
    ///    },
    ///    "houseNumber": {
    ///      "description": "house number",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "description": "unique ID of the location",
    ///      "type": "string"
    ///    },
    ///    "lat": {
    ///      "description": "latitude",
    ///      "type": "number"
    ///    },
    ///    "level": {
    ///      "description": "level according to OpenStreetMap\n(at the moment
    /// only for public transport)\n",
    ///      "type": "number"
    ///    },
    ///    "lon": {
    ///      "description": "longitude",
    ///      "type": "number"
    ///    },
    ///    "name": {
    ///      "description": "name of the location (transit stop / PoI /
    /// address)",
    ///      "type": "string"
    ///    },
    ///    "score": {
    ///      "description": "score according to the internal scoring system (the
    /// scoring algorithm might change in the future)",
    ///      "type": "number"
    ///    },
    ///    "street": {
    ///      "description": "street name",
    ///      "type": "string"
    ///    },
    ///    "tokens": {
    ///      "description": "list of non-overlapping tokens that were matched",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Token"
    ///      }
    ///    },
    ///    "type": {
    ///      "$ref": "#/components/schemas/LocationType"
    ///    },
    ///    "tz": {
    ///      "description": "timezone name (e.g. \"Europe/Berlin\")",
    ///      "type": "string"
    ///    },
    ///    "zip": {
    ///      "description": "zip code",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Match {
        ///list of areas
        pub areas: ::std::vec::Vec<Area>,
        ///house number
        #[serde(
            rename = "houseNumber",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub house_number: ::std::option::Option<::std::string::String>,
        ///unique ID of the location
        pub id: ::std::string::String,
        pub lat: f64,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub level: ::std::option::Option<f64>,
        pub lon: f64,
        ///name of the location (transit stop / PoI / address)
        pub name: ::std::string::String,
        pub score: f64,
        ///street name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub street: ::std::option::Option<::std::string::String>,
        ///list of non-overlapping tokens that were matched
        pub tokens: ::std::vec::Vec<Token>,
        #[serde(rename = "type")]
        pub type_: LocationType,
        ///timezone name (e.g. "Europe/Berlin")
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub tz: ::std::option::Option<::std::string::String>,
        ///zip code
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub zip: ::std::option::Option<::std::string::String>,
    }

    impl ::std::convert::From<&Match> for Match {
        fn from(value: &Match) -> Self {
            value.clone()
        }
    }

    impl Match {
        pub fn builder() -> builder::Match {
            Default::default()
        }
    }

    ///# Street modes
    ///
    ///  - `WALK`
    ///  - `BIKE`
    ///  - `RENTAL` Experimental. Expect unannounced breaking changes (without
    ///    version bumps) for all parameters and returned structs.
    ///  - `CAR`
    ///  - `CAR_PARKING` Experimental. Expect unannounced breaking changes
    ///    (without version bumps) for all parameters and returned structs.
    ///  - `CAR_DROPOFF` Experimental. Expect unannounced breaking changes
    ///    (without version bumps) for all perameters and returned structs.
    ///  - `ODM` on-demand taxis from the Prima+ÖV Project
    ///  - `FLEX` flexible transports
    ///
    ///# Transit modes
    ///
    ///  - `TRANSIT`: translates to
    ///    `RAIL,TRAM,BUS,FERRY,AIRPLANE,COACH,CABLE_CAR,FUNICULAR,AREAL_LIFT,
    ///    OTHER`
    ///  - `TRAM`: trams
    ///  - `SUBWAY`: subway trains
    ///  - `FERRY`: ferries
    ///  - `AIRPLANE`: airline flights
    ///  - `BUS`: short distance buses (does not include `COACH`)
    ///  - `COACH`: long distance buses (does not include `BUS`)
    ///  - `RAIL`: translates to
    ///    `HIGHSPEED_RAIL,LONG_DISTANCE,NIGHT_RAIL,REGIONAL_RAIL,
    ///    REGIONAL_FAST_RAIL,METRO,SUBWAY`
    ///  - `METRO`: metro trains
    ///  - `HIGHSPEED_RAIL`: long distance high speed trains (e.g. TGV)
    ///  - `LONG_DISTANCE`: long distance inter city trains
    ///  - `NIGHT_RAIL`: long distance night trains
    ///  - `REGIONAL_FAST_RAIL`: regional express routes that skip low traffic
    ///    stops to be faster
    ///  - `REGIONAL_RAIL`: regional train
    ///  - `CABLE_CAR`: Cable tram. Used for street-level rail cars where the
    ///    cable runs beneath the vehicle (e.g., cable car in San Francisco).
    ///  - `FUNICULAR`: Funicular. Any rail system designed for steep inclines.
    ///  - `AREAL_LIFT`: Aerial lift, suspended cable car (e.g., gondola lift,
    ///    aerial tramway). Cable transport where cabins, cars, gondolas or open
    ///    chairs are suspended by means of one or more cables.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "# Street modes\n\n  - `WALK`\n  - `BIKE`\n  - `RENTAL`
    /// Experimental. Expect unannounced breaking changes (without version
    /// bumps) for all parameters and returned structs.\n  - `CAR`\n  -
    /// `CAR_PARKING` Experimental. Expect unannounced breaking changes (without
    /// version bumps) for all parameters and returned structs.\n  -
    /// `CAR_DROPOFF` Experimental. Expect unannounced breaking changes (without
    /// version bumps) for all perameters and returned structs.\n  - `ODM`
    /// on-demand taxis from the Prima+ÖV Project\n  - `FLEX` flexible
    /// transports\n\n# Transit modes\n\n  - `TRANSIT`: translates to
    /// `RAIL,TRAM,BUS,FERRY,AIRPLANE,COACH,CABLE_CAR,FUNICULAR,AREAL_LIFT,
    /// OTHER`\n  - `TRAM`: trams\n  - `SUBWAY`: subway trains\n  - `FERRY`:
    /// ferries\n  - `AIRPLANE`: airline flights\n  - `BUS`: short distance
    /// buses (does not include `COACH`)\n  - `COACH`: long distance buses (does
    /// not include `BUS`)\n  - `RAIL`: translates to
    /// `HIGHSPEED_RAIL,LONG_DISTANCE,NIGHT_RAIL,REGIONAL_RAIL,
    /// REGIONAL_FAST_RAIL,METRO,SUBWAY`\n  - `METRO`: metro trains \n  -
    /// `HIGHSPEED_RAIL`: long distance high speed trains (e.g. TGV)\n  -
    /// `LONG_DISTANCE`: long distance inter city trains\n  - `NIGHT_RAIL`: long
    /// distance night trains\n  - `REGIONAL_FAST_RAIL`: regional express routes
    /// that skip low traffic stops to be faster\n  - `REGIONAL_RAIL`: regional
    /// train\n  - `CABLE_CAR`: Cable tram. Used for street-level rail cars
    /// where the cable runs beneath the vehicle (e.g., cable car in San
    /// Francisco).\n  - `FUNICULAR`: Funicular. Any rail system designed for
    /// steep inclines.\n  - `AREAL_LIFT`: Aerial lift, suspended cable car
    /// (e.g., gondola lift, aerial tramway). Cable transport where cabins,
    /// cars, gondolas or open chairs are suspended by means of one or more
    /// cables.\n",
    ///  "type": "string",
    ///  "enum": [
    ///    "WALK",
    ///    "BIKE",
    ///    "RENTAL",
    ///    "CAR",
    ///    "CAR_PARKING",
    ///    "CAR_DROPOFF",
    ///    "ODM",
    ///    "FLEX",
    ///    "TRANSIT",
    ///    "TRAM",
    ///    "SUBWAY",
    ///    "FERRY",
    ///    "AIRPLANE",
    ///    "METRO",
    ///    "BUS",
    ///    "COACH",
    ///    "RAIL",
    ///    "HIGHSPEED_RAIL",
    ///    "LONG_DISTANCE",
    ///    "NIGHT_RAIL",
    ///    "REGIONAL_FAST_RAIL",
    ///    "REGIONAL_RAIL",
    ///    "CABLE_CAR",
    ///    "FUNICULAR",
    ///    "AREAL_LIFT",
    ///    "OTHER"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum Mode {
        #[serde(rename = "WALK")]
        Walk,
        #[serde(rename = "BIKE")]
        Bike,
        #[serde(rename = "RENTAL")]
        Rental,
        #[serde(rename = "CAR")]
        Car,
        #[serde(rename = "CAR_PARKING")]
        CarParking,
        #[serde(rename = "CAR_DROPOFF")]
        CarDropoff,
        #[serde(rename = "ODM")]
        Odm,
        #[serde(rename = "FLEX")]
        Flex,
        #[serde(rename = "TRANSIT")]
        Transit,
        #[serde(rename = "TRAM")]
        Tram,
        #[serde(rename = "SUBWAY")]
        Subway,
        #[serde(rename = "FERRY")]
        Ferry,
        #[serde(rename = "AIRPLANE")]
        Airplane,
        #[serde(rename = "METRO")]
        Metro,
        #[serde(rename = "BUS")]
        Bus,
        #[serde(rename = "COACH")]
        Coach,
        #[serde(rename = "RAIL")]
        Rail,
        #[serde(rename = "HIGHSPEED_RAIL")]
        HighspeedRail,
        #[serde(rename = "LONG_DISTANCE")]
        LongDistance,
        #[serde(rename = "NIGHT_RAIL")]
        NightRail,
        #[serde(rename = "REGIONAL_FAST_RAIL")]
        RegionalFastRail,
        #[serde(rename = "REGIONAL_RAIL")]
        RegionalRail,
        #[serde(rename = "CABLE_CAR")]
        CableCar,
        #[serde(rename = "FUNICULAR")]
        Funicular,
        #[serde(rename = "AREAL_LIFT")]
        ArealLift,
        #[serde(rename = "OTHER")]
        Other,
    }

    impl ::std::convert::From<&Self> for Mode {
        fn from(value: &Mode) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for Mode {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Walk => f.write_str("WALK"),
                Self::Bike => f.write_str("BIKE"),
                Self::Rental => f.write_str("RENTAL"),
                Self::Car => f.write_str("CAR"),
                Self::CarParking => f.write_str("CAR_PARKING"),
                Self::CarDropoff => f.write_str("CAR_DROPOFF"),
                Self::Odm => f.write_str("ODM"),
                Self::Flex => f.write_str("FLEX"),
                Self::Transit => f.write_str("TRANSIT"),
                Self::Tram => f.write_str("TRAM"),
                Self::Subway => f.write_str("SUBWAY"),
                Self::Ferry => f.write_str("FERRY"),
                Self::Airplane => f.write_str("AIRPLANE"),
                Self::Metro => f.write_str("METRO"),
                Self::Bus => f.write_str("BUS"),
                Self::Coach => f.write_str("COACH"),
                Self::Rail => f.write_str("RAIL"),
                Self::HighspeedRail => f.write_str("HIGHSPEED_RAIL"),
                Self::LongDistance => f.write_str("LONG_DISTANCE"),
                Self::NightRail => f.write_str("NIGHT_RAIL"),
                Self::RegionalFastRail => f.write_str("REGIONAL_FAST_RAIL"),
                Self::RegionalRail => f.write_str("REGIONAL_RAIL"),
                Self::CableCar => f.write_str("CABLE_CAR"),
                Self::Funicular => f.write_str("FUNICULAR"),
                Self::ArealLift => f.write_str("AREAL_LIFT"),
                Self::Other => f.write_str("OTHER"),
            }
        }
    }

    impl ::std::str::FromStr for Mode {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "WALK" => Ok(Self::Walk),
                "BIKE" => Ok(Self::Bike),
                "RENTAL" => Ok(Self::Rental),
                "CAR" => Ok(Self::Car),
                "CAR_PARKING" => Ok(Self::CarParking),
                "CAR_DROPOFF" => Ok(Self::CarDropoff),
                "ODM" => Ok(Self::Odm),
                "FLEX" => Ok(Self::Flex),
                "TRANSIT" => Ok(Self::Transit),
                "TRAM" => Ok(Self::Tram),
                "SUBWAY" => Ok(Self::Subway),
                "FERRY" => Ok(Self::Ferry),
                "AIRPLANE" => Ok(Self::Airplane),
                "METRO" => Ok(Self::Metro),
                "BUS" => Ok(Self::Bus),
                "COACH" => Ok(Self::Coach),
                "RAIL" => Ok(Self::Rail),
                "HIGHSPEED_RAIL" => Ok(Self::HighspeedRail),
                "LONG_DISTANCE" => Ok(Self::LongDistance),
                "NIGHT_RAIL" => Ok(Self::NightRail),
                "REGIONAL_FAST_RAIL" => Ok(Self::RegionalFastRail),
                "REGIONAL_RAIL" => Ok(Self::RegionalRail),
                "CABLE_CAR" => Ok(Self::CableCar),
                "FUNICULAR" => Ok(Self::Funicular),
                "AREAL_LIFT" => Ok(Self::ArealLift),
                "OTHER" => Ok(Self::Other),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for Mode {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for Mode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for Mode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///Different accessibility profiles for pedestrians.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Different accessibility profiles for pedestrians.",
    ///  "type": "string",
    ///  "enum": [
    ///    "FOOT",
    ///    "WHEELCHAIR"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum PedestrianProfile {
        #[serde(rename = "FOOT")]
        Foot,
        #[serde(rename = "WHEELCHAIR")]
        Wheelchair,
    }

    impl ::std::convert::From<&Self> for PedestrianProfile {
        fn from(value: &PedestrianProfile) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for PedestrianProfile {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Foot => f.write_str("FOOT"),
                Self::Wheelchair => f.write_str("WHEELCHAIR"),
            }
        }
    }

    impl ::std::str::FromStr for PedestrianProfile {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "FOOT" => Ok(Self::Foot),
                "WHEELCHAIR" => Ok(Self::Wheelchair),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for PedestrianProfile {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for PedestrianProfile {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for PedestrianProfile {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    /// - `NORMAL` - entry/exit is possible normally
    /// - `NOT_ALLOWED` - entry/exit is not allowed
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "- `NORMAL` - entry/exit is possible normally\n-
    /// `NOT_ALLOWED` - entry/exit is not allowed\n",
    ///  "type": "string",
    ///  "enum": [
    ///    "NORMAL",
    ///    "NOT_ALLOWED"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum PickupDropoffType {
        #[serde(rename = "NORMAL")]
        Normal,
        #[serde(rename = "NOT_ALLOWED")]
        NotAllowed,
    }

    impl ::std::convert::From<&Self> for PickupDropoffType {
        fn from(value: &PickupDropoffType) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for PickupDropoffType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Normal => f.write_str("NORMAL"),
                Self::NotAllowed => f.write_str("NOT_ALLOWED"),
            }
        }
    }

    impl ::std::str::FromStr for PickupDropoffType {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "NORMAL" => Ok(Self::Normal),
                "NOT_ALLOWED" => Ok(Self::NotAllowed),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for PickupDropoffType {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for PickupDropoffType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for PickupDropoffType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`Place`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "lat",
    ///    "level",
    ///    "lon",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "alerts": {
    ///      "description": "Alerts for this stop.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Alert"
    ///      }
    ///    },
    ///    "arrival": {
    ///      "description": "arrival time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "cancelled": {
    ///      "description": "Whether this stop is cancelled due to the realtime
    /// situation.",
    ///      "type": "boolean"
    ///    },
    ///    "departure": {
    ///      "description": "departure time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "description": {
    ///      "description": "description of the location that provides more
    /// detailed information",
    ///      "type": "string"
    ///    },
    ///    "dropoffType": {
    ///      "$ref": "#/components/schemas/PickupDropoffType"
    ///    },
    ///    "flex": {
    ///      "description": "for `FLEX` transports, the flex location area or
    /// location group name",
    ///      "type": "string"
    ///    },
    ///    "flexEndPickupDropOffWindow": {
    ///      "description": "Time that on-demand service ends",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "flexId": {
    ///      "description": "for `FLEX` transports, the flex location area ID or
    /// location group ID",
    ///      "type": "string"
    ///    },
    ///    "flexStartPickupDropOffWindow": {
    ///      "description": "Time that on-demand service becomes available",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "lat": {
    ///      "description": "latitude",
    ///      "type": "number"
    ///    },
    ///    "level": {
    ///      "description": "level according to OpenStreetMap",
    ///      "type": "number"
    ///    },
    ///    "lon": {
    ///      "description": "longitude",
    ///      "type": "number"
    ///    },
    ///    "name": {
    ///      "description": "name of the transit stop / PoI / address",
    ///      "type": "string"
    ///    },
    ///    "pickupType": {
    ///      "$ref": "#/components/schemas/PickupDropoffType"
    ///    },
    ///    "scheduledArrival": {
    ///      "description": "scheduled arrival time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "scheduledDeparture": {
    ///      "description": "scheduled departure time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "scheduledTrack": {
    ///      "description": "scheduled track from the static schedule timetable
    /// dataset",
    ///      "type": "string"
    ///    },
    ///    "stopId": {
    ///      "description": "The ID of the stop. This is often something that
    /// users don't care about.",
    ///      "type": "string"
    ///    },
    ///    "track": {
    ///      "description": "The current track/platform information, updated
    /// with real-time updates if available. \nCan be missing if neither
    /// real-time updates nor the schedule timetable contains track
    /// information.\n",
    ///      "type": "string"
    ///    },
    ///    "tz": {
    ///      "description": "timezone name (e.g. \"Europe/Berlin\")",
    ///      "type": "string"
    ///    },
    ///    "vertexType": {
    ///      "$ref": "#/components/schemas/VertexType"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Place {
        ///Alerts for this stop.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        pub alerts: ::std::vec::Vec<Alert>,
        ///arrival time
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub arrival: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
        ///Whether this stop is cancelled due to the realtime situation.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub cancelled: ::std::option::Option<bool>,
        ///departure time
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub departure: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
        ///description of the location that provides more detailed information
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "dropoffType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dropoff_type: ::std::option::Option<PickupDropoffType>,
        ///for `FLEX` transports, the flex location area or location group name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub flex: ::std::option::Option<::std::string::String>,
        ///Time that on-demand service ends
        #[serde(
            rename = "flexEndPickupDropOffWindow",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub flex_end_pickup_drop_off_window:
            ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
        ///for `FLEX` transports, the flex location area ID or location group
        /// ID
        #[serde(
            rename = "flexId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub flex_id: ::std::option::Option<::std::string::String>,
        ///Time that on-demand service becomes available
        #[serde(
            rename = "flexStartPickupDropOffWindow",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub flex_start_pickup_drop_off_window:
            ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
        pub lat: f64,
        pub level: f64,
        pub lon: f64,
        ///name of the transit stop / PoI / address
        pub name: ::std::string::String,
        #[serde(
            rename = "pickupType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub pickup_type: ::std::option::Option<PickupDropoffType>,
        ///scheduled arrival time
        #[serde(
            rename = "scheduledArrival",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub scheduled_arrival: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
        ///scheduled departure time
        #[serde(
            rename = "scheduledDeparture",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub scheduled_departure: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
        ///scheduled track from the static schedule timetable dataset
        #[serde(
            rename = "scheduledTrack",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub scheduled_track: ::std::option::Option<::std::string::String>,
        ///The ID of the stop. This is often something that users don't care
        /// about.
        #[serde(
            rename = "stopId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub stop_id: ::std::option::Option<::std::string::String>,
        ///The current track/platform information, updated with real-time
        /// updates if available. Can be missing if neither real-time
        /// updates nor the schedule timetable contains track information.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub track: ::std::option::Option<::std::string::String>,
        ///timezone name (e.g. "Europe/Berlin")
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub tz: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "vertexType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub vertex_type: ::std::option::Option<VertexType>,
    }

    impl ::std::convert::From<&Place> for Place {
        fn from(value: &Place) -> Self {
            value.clone()
        }
    }

    impl Place {
        pub fn builder() -> builder::Place {
            Default::default()
        }
    }

    ///`PlanAlgorithm`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "default": "RAPTOR",
    ///  "type": "string",
    ///  "enum": [
    ///    "RAPTOR",
    ///    "TB"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum PlanAlgorithm {
        #[serde(rename = "RAPTOR")]
        Raptor,
        #[serde(rename = "TB")]
        Tb,
    }

    impl ::std::convert::From<&Self> for PlanAlgorithm {
        fn from(value: &PlanAlgorithm) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for PlanAlgorithm {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Raptor => f.write_str("RAPTOR"),
                Self::Tb => f.write_str("TB"),
            }
        }
    }

    impl ::std::str::FromStr for PlanAlgorithm {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "RAPTOR" => Ok(Self::Raptor),
                "TB" => Ok(Self::Tb),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for PlanAlgorithm {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for PlanAlgorithm {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for PlanAlgorithm {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::default::Default for PlanAlgorithm {
        fn default() -> Self {
            PlanAlgorithm::Raptor
        }
    }

    ///`PlanResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "debugOutput",
    ///    "direct",
    ///    "from",
    ///    "itineraries",
    ///    "nextPageCursor",
    ///    "previousPageCursor",
    ///    "requestParameters",
    ///    "to"
    ///  ],
    ///  "properties": {
    ///    "debugOutput": {
    ///      "description": "debug statistics",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "integer"
    ///      }
    ///    },
    ///    "direct": {
    ///      "description": "Direct trips by `WALK`, `BIKE`, `CAR`, etc. without
    /// time-dependency.\nThe starting time (`arriveBy=false`) / arrival time
    /// (`arriveBy=true`) is always the queried `time` parameter (set to
    /// \\\"now\\\" if not set).\nBut all `direct` connections are meant to be
    /// independent of absolute times.\n",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Itinerary"
    ///      }
    ///    },
    ///    "from": {
    ///      "$ref": "#/components/schemas/Place"
    ///    },
    ///    "itineraries": {
    ///      "description": "list of itineraries",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Itinerary"
    ///      }
    ///    },
    ///    "nextPageCursor": {
    ///      "description": "Use the cursor to get the next page of results.
    /// Insert the cursor into the request and post it to get the next
    /// page.\nThe next page is a set of itineraries departing AFTER the last
    /// itinerary in this result.\n",
    ///      "type": "string"
    ///    },
    ///    "previousPageCursor": {
    ///      "description": "Use the cursor to get the previous page of results.
    /// Insert the cursor into the request and post it to get the previous
    /// page.\nThe previous page is a set of itineraries departing BEFORE the
    /// first itinerary in the result for a depart after search. When using the
    /// default sort order the previous set of itineraries is inserted before
    /// the current result.\n",
    ///      "type": "string"
    ///    },
    ///    "requestParameters": {
    ///      "description": "the routing query",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "to": {
    ///      "$ref": "#/components/schemas/Place"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct PlanResponse {
        ///debug statistics
        #[serde(rename = "debugOutput")]
        pub debug_output: ::std::collections::HashMap<::std::string::String, i64>,
        ///Direct trips by `WALK`, `BIKE`, `CAR`, etc. without time-dependency.
        ///The starting time (`arriveBy=false`) / arrival time
        /// (`arriveBy=true`) is always the queried `time` parameter (set to
        /// \"now\" if not set). But all `direct` connections are meant
        /// to be independent of absolute times.
        pub direct: ::std::vec::Vec<Itinerary>,
        pub from: Place,
        ///list of itineraries
        pub itineraries: ::std::vec::Vec<Itinerary>,
        ///Use the cursor to get the next page of results. Insert the cursor
        /// into the request and post it to get the next page.
        /// The next page is a set of itineraries departing AFTER the last
        /// itinerary in this result.
        #[serde(rename = "nextPageCursor")]
        pub next_page_cursor: ::std::string::String,
        ///Use the cursor to get the previous page of results. Insert the
        /// cursor into the request and post it to get the previous page.
        /// The previous page is a set of itineraries departing BEFORE the first
        /// itinerary in the result for a depart after search. When using the
        /// default sort order the previous set of itineraries is inserted
        /// before the current result.
        #[serde(rename = "previousPageCursor")]
        pub previous_page_cursor: ::std::string::String,
        ///the routing query
        #[serde(rename = "requestParameters")]
        pub request_parameters:
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        pub to: Place,
    }

    impl ::std::convert::From<&PlanResponse> for PlanResponse {
        fn from(value: &PlanResponse) -> Self {
            value.clone()
        }
    }

    impl PlanResponse {
        pub fn builder() -> builder::PlanResponse {
            Default::default()
        }
    }

    ///Object containing all reachable places by One-to-All search
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Object containing all reachable places by One-to-All
    /// search",
    ///  "type": "object",
    ///  "properties": {
    ///    "all": {
    ///      "description": "List of locations reachable by One-to-All",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ReachablePlace"
    ///      }
    ///    },
    ///    "one": {
    ///      "$ref": "#/components/schemas/Place"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Reachable {
        ///List of locations reachable by One-to-All
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        pub all: ::std::vec::Vec<ReachablePlace>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub one: ::std::option::Option<Place>,
    }

    impl ::std::convert::From<&Reachable> for Reachable {
        fn from(value: &Reachable) -> Self {
            value.clone()
        }
    }

    impl ::std::default::Default for Reachable {
        fn default() -> Self {
            Self {
                all: Default::default(),
                one: Default::default(),
            }
        }
    }

    impl Reachable {
        pub fn builder() -> builder::Reachable {
            Default::default()
        }
    }

    ///Place reachable by One-to-All
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Place reachable by One-to-All",
    ///  "type": "object",
    ///  "properties": {
    ///    "duration": {
    ///      "description": "Total travel duration",
    ///      "type": "integer"
    ///    },
    ///    "k": {
    ///      "description": "k is the smallest number, for which a journey with
    /// the shortest duration and at most k-1 transfers exist.\nYou can think of
    /// k as the number of connections used.\n\nIn more detail:\n\nk=0: No
    /// connection, e.g. for the one location\nk=1: Direct connection\nk=2:
    /// Connection with 1 transfer\n",
    ///      "type": "integer"
    ///    },
    ///    "place": {
    ///      "$ref": "#/components/schemas/Place"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct ReachablePlace {
        ///Total travel duration
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub duration: ::std::option::Option<i64>,
        ///k is the smallest number, for which a journey with the shortest
        /// duration and at most k-1 transfers exist. You can think of k
        /// as the number of connections used.
        ///
        ///In more detail:
        ///
        ///k=0: No connection, e.g. for the one location
        ///k=1: Direct connection
        ///k=2: Connection with 1 transfer
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub k: ::std::option::Option<i64>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub place: ::std::option::Option<Place>,
    }

    impl ::std::convert::From<&ReachablePlace> for ReachablePlace {
        fn from(value: &ReachablePlace) -> Self {
            value.clone()
        }
    }

    impl ::std::default::Default for ReachablePlace {
        fn default() -> Self {
            Self {
                duration: Default::default(),
                k: Default::default(),
                place: Default::default(),
            }
        }
    }

    impl ReachablePlace {
        pub fn builder() -> builder::ReachablePlace {
            Default::default()
        }
    }

    ///Vehicle rental
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Vehicle rental",
    ///  "type": "object",
    ///  "required": [
    ///    "systemId"
    ///  ],
    ///  "properties": {
    ///    "formFactor": {
    ///      "$ref": "#/components/schemas/RentalFormFactor"
    ///    },
    ///    "fromStationName": {
    ///      "description": "Name of the station where the vehicle is picked up
    /// (empty for free floating vehicles)",
    ///      "type": "string"
    ///    },
    ///    "propulsionType": {
    ///      "$ref": "#/components/schemas/RentalPropulsionType"
    ///    },
    ///    "rentalUriAndroid": {
    ///      "description": "Rental URI for Android (deep link to the specific
    /// station or vehicle)",
    ///      "type": "string"
    ///    },
    ///    "rentalUriIOS": {
    ///      "description": "Rental URI for iOS (deep link to the specific
    /// station or vehicle)",
    ///      "type": "string"
    ///    },
    ///    "rentalUriWeb": {
    ///      "description": "Rental URI for web (deep link to the specific
    /// station or vehicle)",
    ///      "type": "string"
    ///    },
    ///    "returnConstraint": {
    ///      "$ref": "#/components/schemas/RentalReturnConstraint"
    ///    },
    ///    "stationName": {
    ///      "description": "Name of the station",
    ///      "type": "string"
    ///    },
    ///    "systemId": {
    ///      "description": "Vehicle share system ID",
    ///      "type": "string"
    ///    },
    ///    "systemName": {
    ///      "description": "Vehicle share system name",
    ///      "type": "string"
    ///    },
    ///    "toStationName": {
    ///      "description": "Name of the station where the vehicle is returned
    /// (empty for free floating vehicles)",
    ///      "type": "string"
    ///    },
    ///    "url": {
    ///      "description": "URL of the vehicle share system",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Rental {
        #[serde(
            rename = "formFactor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub form_factor: ::std::option::Option<RentalFormFactor>,
        ///Name of the station where the vehicle is picked up (empty for free
        /// floating vehicles)
        #[serde(
            rename = "fromStationName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub from_station_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "propulsionType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub propulsion_type: ::std::option::Option<RentalPropulsionType>,
        ///Rental URI for Android (deep link to the specific station or
        /// vehicle)
        #[serde(
            rename = "rentalUriAndroid",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub rental_uri_android: ::std::option::Option<::std::string::String>,
        ///Rental URI for iOS (deep link to the specific station or vehicle)
        #[serde(
            rename = "rentalUriIOS",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub rental_uri_ios: ::std::option::Option<::std::string::String>,
        ///Rental URI for web (deep link to the specific station or vehicle)
        #[serde(
            rename = "rentalUriWeb",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub rental_uri_web: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "returnConstraint",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub return_constraint: ::std::option::Option<RentalReturnConstraint>,
        ///Name of the station
        #[serde(
            rename = "stationName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub station_name: ::std::option::Option<::std::string::String>,
        ///Vehicle share system ID
        #[serde(rename = "systemId")]
        pub system_id: ::std::string::String,
        ///Vehicle share system name
        #[serde(
            rename = "systemName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub system_name: ::std::option::Option<::std::string::String>,
        ///Name of the station where the vehicle is returned (empty for free
        /// floating vehicles)
        #[serde(
            rename = "toStationName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub to_station_name: ::std::option::Option<::std::string::String>,
        ///URL of the vehicle share system
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub url: ::std::option::Option<::std::string::String>,
    }

    impl ::std::convert::From<&Rental> for Rental {
        fn from(value: &Rental) -> Self {
            value.clone()
        }
    }

    impl Rental {
        pub fn builder() -> builder::Rental {
            Default::default()
        }
    }

    ///`RentalFormFactor`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "BICYCLE",
    ///    "CARGO_BICYCLE",
    ///    "CAR",
    ///    "MOPED",
    ///    "SCOOTER_STANDING",
    ///    "SCOOTER_SEATED",
    ///    "OTHER"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum RentalFormFactor {
        #[serde(rename = "BICYCLE")]
        Bicycle,
        #[serde(rename = "CARGO_BICYCLE")]
        CargoBicycle,
        #[serde(rename = "CAR")]
        Car,
        #[serde(rename = "MOPED")]
        Moped,
        #[serde(rename = "SCOOTER_STANDING")]
        ScooterStanding,
        #[serde(rename = "SCOOTER_SEATED")]
        ScooterSeated,
        #[serde(rename = "OTHER")]
        Other,
    }

    impl ::std::convert::From<&Self> for RentalFormFactor {
        fn from(value: &RentalFormFactor) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for RentalFormFactor {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Bicycle => f.write_str("BICYCLE"),
                Self::CargoBicycle => f.write_str("CARGO_BICYCLE"),
                Self::Car => f.write_str("CAR"),
                Self::Moped => f.write_str("MOPED"),
                Self::ScooterStanding => f.write_str("SCOOTER_STANDING"),
                Self::ScooterSeated => f.write_str("SCOOTER_SEATED"),
                Self::Other => f.write_str("OTHER"),
            }
        }
    }

    impl ::std::str::FromStr for RentalFormFactor {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "BICYCLE" => Ok(Self::Bicycle),
                "CARGO_BICYCLE" => Ok(Self::CargoBicycle),
                "CAR" => Ok(Self::Car),
                "MOPED" => Ok(Self::Moped),
                "SCOOTER_STANDING" => Ok(Self::ScooterStanding),
                "SCOOTER_SEATED" => Ok(Self::ScooterSeated),
                "OTHER" => Ok(Self::Other),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for RentalFormFactor {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for RentalFormFactor {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for RentalFormFactor {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`RentalPropulsionType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "HUMAN",
    ///    "ELECTRIC_ASSIST",
    ///    "ELECTRIC",
    ///    "COMBUSTION",
    ///    "COMBUSTION_DIESEL",
    ///    "HYBRID",
    ///    "PLUG_IN_HYBRID",
    ///    "HYDROGEN_FUEL_CELL"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum RentalPropulsionType {
        #[serde(rename = "HUMAN")]
        Human,
        #[serde(rename = "ELECTRIC_ASSIST")]
        ElectricAssist,
        #[serde(rename = "ELECTRIC")]
        Electric,
        #[serde(rename = "COMBUSTION")]
        Combustion,
        #[serde(rename = "COMBUSTION_DIESEL")]
        CombustionDiesel,
        #[serde(rename = "HYBRID")]
        Hybrid,
        #[serde(rename = "PLUG_IN_HYBRID")]
        PlugInHybrid,
        #[serde(rename = "HYDROGEN_FUEL_CELL")]
        HydrogenFuelCell,
    }

    impl ::std::convert::From<&Self> for RentalPropulsionType {
        fn from(value: &RentalPropulsionType) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for RentalPropulsionType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Human => f.write_str("HUMAN"),
                Self::ElectricAssist => f.write_str("ELECTRIC_ASSIST"),
                Self::Electric => f.write_str("ELECTRIC"),
                Self::Combustion => f.write_str("COMBUSTION"),
                Self::CombustionDiesel => f.write_str("COMBUSTION_DIESEL"),
                Self::Hybrid => f.write_str("HYBRID"),
                Self::PlugInHybrid => f.write_str("PLUG_IN_HYBRID"),
                Self::HydrogenFuelCell => f.write_str("HYDROGEN_FUEL_CELL"),
            }
        }
    }

    impl ::std::str::FromStr for RentalPropulsionType {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "HUMAN" => Ok(Self::Human),
                "ELECTRIC_ASSIST" => Ok(Self::ElectricAssist),
                "ELECTRIC" => Ok(Self::Electric),
                "COMBUSTION" => Ok(Self::Combustion),
                "COMBUSTION_DIESEL" => Ok(Self::CombustionDiesel),
                "HYBRID" => Ok(Self::Hybrid),
                "PLUG_IN_HYBRID" => Ok(Self::PlugInHybrid),
                "HYDROGEN_FUEL_CELL" => Ok(Self::HydrogenFuelCell),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for RentalPropulsionType {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for RentalPropulsionType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for RentalPropulsionType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`RentalReturnConstraint`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "NONE",
    ///    "ANY_STATION",
    ///    "ROUNDTRIP_STATION"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum RentalReturnConstraint {
        #[serde(rename = "NONE")]
        None,
        #[serde(rename = "ANY_STATION")]
        AnyStation,
        #[serde(rename = "ROUNDTRIP_STATION")]
        RoundtripStation,
    }

    impl ::std::convert::From<&Self> for RentalReturnConstraint {
        fn from(value: &RentalReturnConstraint) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for RentalReturnConstraint {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::None => f.write_str("NONE"),
                Self::AnyStation => f.write_str("ANY_STATION"),
                Self::RoundtripStation => f.write_str("ROUNDTRIP_STATION"),
            }
        }
    }

    impl ::std::str::FromStr for RentalReturnConstraint {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "NONE" => Ok(Self::None),
                "ANY_STATION" => Ok(Self::AnyStation),
                "ROUNDTRIP_STATION" => Ok(Self::RoundtripStation),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for RentalReturnConstraint {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for RentalReturnConstraint {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for RentalReturnConstraint {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`RiderCategory`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "isDefaultFareCategory",
    ///    "riderCategoryName"
    ///  ],
    ///  "properties": {
    ///    "eligibilityUrl": {
    ///      "description": "URL to a web page providing detailed information
    /// about the rider category and/or its eligibility criteria.",
    ///      "type": "string"
    ///    },
    ///    "isDefaultFareCategory": {
    ///      "description": "Specifies if this category should be considered the
    /// default (i.e. the main category displayed to riders).",
    ///      "type": "boolean"
    ///    },
    ///    "riderCategoryName": {
    ///      "description": "Rider category name as displayed to the rider.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct RiderCategory {
        ///URL to a web page providing detailed information about the rider
        /// category and/or its eligibility criteria.
        #[serde(
            rename = "eligibilityUrl",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub eligibility_url: ::std::option::Option<::std::string::String>,
        ///Specifies if this category should be considered the default (i.e.
        /// the main category displayed to riders).
        #[serde(rename = "isDefaultFareCategory")]
        pub is_default_fare_category: bool,
        ///Rider category name as displayed to the rider.
        #[serde(rename = "riderCategoryName")]
        pub rider_category_name: ::std::string::String,
    }

    impl ::std::convert::From<&RiderCategory> for RiderCategory {
        fn from(value: &RiderCategory) -> Self {
            value.clone()
        }
    }

    impl RiderCategory {
        pub fn builder() -> builder::RiderCategory {
            Default::default()
        }
    }

    ///`StepInstruction`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "area",
    ///    "distance",
    ///    "exit",
    ///    "fromLevel",
    ///    "polyline",
    ///    "relativeDirection",
    ///    "stayOn",
    ///    "streetName",
    ///    "toLevel"
    ///  ],
    ///  "properties": {
    ///    "accessRestriction": {
    ///      "description": "Experimental. Indicates whether access to this part of the route is restricted.\nSee: https://wiki.openstreetmap.org/wiki/Conditional_restrictions\n",
    ///      "type": "string"
    ///    },
    ///    "area": {
    ///      "description": "Not implemented!\nThis step is on an open area,
    /// such as a plaza or train platform,\nand thus the directions should say
    /// something like \"cross\"\n",
    ///      "type": "boolean"
    ///    },
    ///    "distance": {
    ///      "description": "The distance in meters that this step takes.",
    ///      "type": "number"
    ///    },
    ///    "elevationDown": {
    ///      "description": "decline in meters across this path segment",
    ///      "type": "integer"
    ///    },
    ///    "elevationUp": {
    ///      "description": "incline in meters across this path segment",
    ///      "type": "integer"
    ///    },
    ///    "exit": {
    ///      "description": "Not implemented!\nWhen exiting a highway or traffic
    /// circle, the exit name/number.\n",
    ///      "type": "string"
    ///    },
    ///    "fromLevel": {
    ///      "description": "level where this segment starts, based on
    /// OpenStreetMap data",
    ///      "type": "number"
    ///    },
    ///    "osmWay": {
    ///      "description": "OpenStreetMap way index",
    ///      "type": "integer"
    ///    },
    ///    "polyline": {
    ///      "$ref": "#/components/schemas/EncodedPolyline"
    ///    },
    ///    "relativeDirection": {
    ///      "$ref": "#/components/schemas/Direction"
    ///    },
    ///    "stayOn": {
    ///      "description": "Not implemented!\nIndicates whether or not a street
    /// changes direction at an intersection.\n",
    ///      "type": "boolean"
    ///    },
    ///    "streetName": {
    ///      "description": "The name of the street.",
    ///      "type": "string"
    ///    },
    ///    "toLevel": {
    ///      "description": "level where this segment starts, based on
    /// OpenStreetMap data",
    ///      "type": "number"
    ///    },
    ///    "toll": {
    ///      "description": "Indicates that a fee must be paid by general
    /// traffic to use a road, road bridge or road tunnel.",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct StepInstruction {
        ///Experimental. Indicates whether access to this part of the route is
        /// restricted. See: https://wiki.openstreetmap.org/wiki/Conditional_restrictions
        #[serde(
            rename = "accessRestriction",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub access_restriction: ::std::option::Option<::std::string::String>,
        ///Not implemented!
        ///This step is on an open area, such as a plaza or train platform,
        ///and thus the directions should say something like "cross"
        pub area: bool,
        pub distance: f64,
        ///decline in meters across this path segment
        #[serde(
            rename = "elevationDown",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub elevation_down: ::std::option::Option<i64>,
        ///incline in meters across this path segment
        #[serde(
            rename = "elevationUp",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub elevation_up: ::std::option::Option<i64>,
        ///Not implemented!
        ///When exiting a highway or traffic circle, the exit name/number.
        pub exit: ::std::string::String,
        #[serde(rename = "fromLevel")]
        pub from_level: f64,
        ///OpenStreetMap way index
        #[serde(
            rename = "osmWay",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub osm_way: ::std::option::Option<i64>,
        pub polyline: EncodedPolyline,
        #[serde(rename = "relativeDirection")]
        pub relative_direction: Direction,
        ///Not implemented!
        ///Indicates whether or not a street changes direction at an
        /// intersection.
        #[serde(rename = "stayOn")]
        pub stay_on: bool,
        ///The name of the street.
        #[serde(rename = "streetName")]
        pub street_name: ::std::string::String,
        #[serde(rename = "toLevel")]
        pub to_level: f64,
        ///Indicates that a fee must be paid by general traffic to use a road,
        /// road bridge or road tunnel.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub toll: ::std::option::Option<bool>,
    }

    impl ::std::convert::From<&StepInstruction> for StepInstruction {
        fn from(value: &StepInstruction) -> Self {
            value.clone()
        }
    }

    impl StepInstruction {
        pub fn builder() -> builder::StepInstruction {
            Default::default()
        }
    }

    ///departure or arrival event at a stop
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "departure or arrival event at a stop",
    ///  "type": "object",
    ///  "required": [
    ///    "agencyId",
    ///    "agencyName",
    ///    "agencyUrl",
    ///    "cancelled",
    ///    "displayName",
    ///    "headsign",
    ///    "mode",
    ///    "pickupDropoffType",
    ///    "place",
    ///    "realTime",
    ///    "routeLongName",
    ///    "routeShortName",
    ///    "source",
    ///    "tripCancelled",
    ///    "tripId",
    ///    "tripShortName",
    ///    "tripTo"
    ///  ],
    ///  "properties": {
    ///    "agencyId": {
    ///      "type": "string"
    ///    },
    ///    "agencyName": {
    ///      "type": "string"
    ///    },
    ///    "agencyUrl": {
    ///      "type": "string"
    ///    },
    ///    "cancelled": {
    ///      "description": "Whether the departure/arrival is cancelled due to
    /// the realtime situation (either because the stop is skipped or because
    /// the entire trip is cancelled).",
    ///      "type": "boolean"
    ///    },
    ///    "displayName": {
    ///      "type": "string"
    ///    },
    ///    "headsign": {
    ///      "description": "The headsign of the bus or train being used.\nFor
    /// non-transit legs, null\n",
    ///      "type": "string"
    ///    },
    ///    "mode": {
    ///      "$ref": "#/components/schemas/Mode"
    ///    },
    ///    "nextStops": {
    ///      "description": "Experimental. Expect unannounced breaking changes
    /// (without version bumps).\n\nStops on the trips after this stop. Returned
    /// only if `fetchStop` is `true` and `arriveBy` is `false`.\n",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Place"
    ///      }
    ///    },
    ///    "pickupDropoffType": {
    ///      "$ref": "#/components/schemas/PickupDropoffType"
    ///    },
    ///    "place": {
    ///      "$ref": "#/components/schemas/Place"
    ///    },
    ///    "previousStops": {
    ///      "description": "Experimental. Expect unannounced breaking changes
    /// (without version bumps).\n\nStops on the trips before this stop.
    /// Returned only if `fetchStop` and `arriveBy` are `true`.\n",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Place"
    ///      }
    ///    },
    ///    "realTime": {
    ///      "description": "Whether there is real-time data about this leg",
    ///      "type": "boolean"
    ///    },
    ///    "routeColor": {
    ///      "type": "string"
    ///    },
    ///    "routeLongName": {
    ///      "type": "string"
    ///    },
    ///    "routeShortName": {
    ///      "type": "string"
    ///    },
    ///    "routeTextColor": {
    ///      "type": "string"
    ///    },
    ///    "routeType": {
    ///      "type": "integer"
    ///    },
    ///    "source": {
    ///      "description": "Filename and line number where this trip is from",
    ///      "type": "string"
    ///    },
    ///    "tripCancelled": {
    ///      "description": "Whether the entire trip is cancelled due to the
    /// realtime situation.",
    ///      "type": "boolean"
    ///    },
    ///    "tripId": {
    ///      "type": "string"
    ///    },
    ///    "tripShortName": {
    ///      "type": "string"
    ///    },
    ///    "tripTo": {
    ///      "$ref": "#/components/schemas/Place"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct StopTime {
        #[serde(rename = "agencyId")]
        pub agency_id: ::std::string::String,
        #[serde(rename = "agencyName")]
        pub agency_name: ::std::string::String,
        #[serde(rename = "agencyUrl")]
        pub agency_url: ::std::string::String,
        ///Whether the departure/arrival is cancelled due to the realtime
        /// situation (either because the stop is skipped or because the entire
        /// trip is cancelled).
        pub cancelled: bool,
        #[serde(rename = "displayName")]
        pub display_name: ::std::string::String,
        ///The headsign of the bus or train being used.
        ///For non-transit legs, null
        pub headsign: ::std::string::String,
        pub mode: Mode,
        ///Experimental. Expect unannounced breaking changes (without version
        /// bumps).
        ///
        ///Stops on the trips after this stop. Returned only if `fetchStop` is
        /// `true` and `arriveBy` is `false`.
        #[serde(
            rename = "nextStops",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        pub next_stops: ::std::vec::Vec<Place>,
        #[serde(rename = "pickupDropoffType")]
        pub pickup_dropoff_type: PickupDropoffType,
        pub place: Place,
        ///Experimental. Expect unannounced breaking changes (without version
        /// bumps).
        ///
        ///Stops on the trips before this stop. Returned only if `fetchStop`
        /// and `arriveBy` are `true`.
        #[serde(
            rename = "previousStops",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        pub previous_stops: ::std::vec::Vec<Place>,
        ///Whether there is real-time data about this leg
        #[serde(rename = "realTime")]
        pub real_time: bool,
        #[serde(
            rename = "routeColor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_color: ::std::option::Option<::std::string::String>,
        #[serde(rename = "routeLongName")]
        pub route_long_name: ::std::string::String,
        #[serde(rename = "routeShortName")]
        pub route_short_name: ::std::string::String,
        #[serde(
            rename = "routeTextColor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_text_color: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "routeType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_type: ::std::option::Option<i64>,
        ///Filename and line number where this trip is from
        pub source: ::std::string::String,
        ///Whether the entire trip is cancelled due to the realtime situation.
        #[serde(rename = "tripCancelled")]
        pub trip_cancelled: bool,
        #[serde(rename = "tripId")]
        pub trip_id: ::std::string::String,
        #[serde(rename = "tripShortName")]
        pub trip_short_name: ::std::string::String,
        #[serde(rename = "tripTo")]
        pub trip_to: Place,
    }

    impl ::std::convert::From<&StopTime> for StopTime {
        fn from(value: &StopTime) -> Self {
            value.clone()
        }
    }

    impl StopTime {
        pub fn builder() -> builder::StopTime {
            Default::default()
        }
    }

    ///`StoptimesDirection`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "EARLIER",
    ///    "LATER"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum StoptimesDirection {
        #[serde(rename = "EARLIER")]
        Earlier,
        #[serde(rename = "LATER")]
        Later,
    }

    impl ::std::convert::From<&Self> for StoptimesDirection {
        fn from(value: &StoptimesDirection) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for StoptimesDirection {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Earlier => f.write_str("EARLIER"),
                Self::Later => f.write_str("LATER"),
            }
        }
    }

    impl ::std::str::FromStr for StoptimesDirection {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "EARLIER" => Ok(Self::Earlier),
                "LATER" => Ok(Self::Later),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for StoptimesDirection {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for StoptimesDirection {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for StoptimesDirection {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    ///`StoptimesResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "nextPageCursor",
    ///    "place",
    ///    "previousPageCursor",
    ///    "stopTimes"
    ///  ],
    ///  "properties": {
    ///    "nextPageCursor": {
    ///      "description": "Use the cursor to get the next page of results.
    /// Insert the cursor into the request and post it to get the next
    /// page.\nThe next page is a set of stop times AFTER the last stop time in
    /// this result.\n",
    ///      "type": "string"
    ///    },
    ///    "place": {
    ///      "$ref": "#/components/schemas/Place"
    ///    },
    ///    "previousPageCursor": {
    ///      "description": "Use the cursor to get the previous page of results.
    /// Insert the cursor into the request and post it to get the previous
    /// page.\nThe previous page is a set of stop times BEFORE the first stop
    /// time in the result.\n",
    ///      "type": "string"
    ///    },
    ///    "stopTimes": {
    ///      "description": "list of stop times",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/StopTime"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct StoptimesResponse {
        ///Use the cursor to get the next page of results. Insert the cursor
        /// into the request and post it to get the next page.
        /// The next page is a set of stop times AFTER the last stop time in
        /// this result.
        #[serde(rename = "nextPageCursor")]
        pub next_page_cursor: ::std::string::String,
        pub place: Place,
        ///Use the cursor to get the previous page of results. Insert the
        /// cursor into the request and post it to get the previous page.
        /// The previous page is a set of stop times BEFORE the first stop time
        /// in the result.
        #[serde(rename = "previousPageCursor")]
        pub previous_page_cursor: ::std::string::String,
        ///list of stop times
        #[serde(rename = "stopTimes")]
        pub stop_times: ::std::vec::Vec<StopTime>,
    }

    impl ::std::convert::From<&StoptimesResponse> for StoptimesResponse {
        fn from(value: &StoptimesResponse) -> Self {
            value.clone()
        }
    }

    impl StoptimesResponse {
        pub fn builder() -> builder::StoptimesResponse {
            Default::default()
        }
    }

    ///A time interval.
    ///The interval is considered active at time t if t is greater than or
    /// equal to the start time and less than the end time.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A time interval.\nThe interval is considered active at
    /// time t if t is greater than or equal to the start time and less than the
    /// end time.\n",
    ///  "type": "object",
    ///  "properties": {
    ///    "end": {
    ///      "description": "If missing, the interval ends at plus infinity.\nIf
    /// a TimeRange is provided, either start or end must be provided - both
    /// fields cannot be empty.\n",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "start": {
    ///      "description": "If missing, the interval starts at minus
    /// infinity.\nIf a TimeRange is provided, either start or end must be
    /// provided - both fields cannot be empty.\n",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct TimeRange {
        ///If missing, the interval ends at plus infinity.
        ///If a TimeRange is provided, either start or end must be provided -
        /// both fields cannot be empty.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub end: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
        ///If missing, the interval starts at minus infinity.
        ///If a TimeRange is provided, either start or end must be provided -
        /// both fields cannot be empty.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub start: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
    }

    impl ::std::convert::From<&TimeRange> for TimeRange {
        fn from(value: &TimeRange) -> Self {
            value.clone()
        }
    }

    impl ::std::default::Default for TimeRange {
        fn default() -> Self {
            Self {
                end: Default::default(),
                start: Default::default(),
            }
        }
    }

    impl TimeRange {
        pub fn builder() -> builder::TimeRange {
            Default::default()
        }
    }

    ///Matched token range (from index, length)
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Matched token range (from index, length)",
    ///  "type": "array",
    ///  "items": {
    ///    "type": "number"
    ///  },
    ///  "maxItems": 2,
    ///  "minItems": 2
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    #[serde(transparent)]
    pub struct Token(pub [f64; 2usize]);
    impl ::std::ops::Deref for Token {
        type Target = [f64; 2usize];
        fn deref(&self) -> &[f64; 2usize] {
            &self.0
        }
    }

    impl ::std::convert::From<Token> for [f64; 2usize] {
        fn from(value: Token) -> Self {
            value.0
        }
    }

    impl ::std::convert::From<&Token> for Token {
        fn from(value: &Token) -> Self {
            value.clone()
        }
    }

    impl ::std::convert::From<[f64; 2usize]> for Token {
        fn from(value: [f64; 2usize]) -> Self {
            Self(value)
        }
    }

    ///transfer from one location to another
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "transfer from one location to another",
    ///  "type": "object",
    ///  "required": [
    ///    "to"
    ///  ],
    ///  "properties": {
    ///    "car": {
    ///      "description": "optional; missing if no path was found with car
    /// routing\ntransfer duration in minutes for the car profile\n",
    ///      "type": "number"
    ///    },
    ///    "default": {
    ///      "description": "optional; missing if the GTFS did not contain a
    /// transfer\ntransfer duration in minutes according to GTFS
    /// (+heuristics)\n",
    ///      "type": "number"
    ///    },
    ///    "foot": {
    ///      "description": "optional; missing if no path was found (timetable /
    /// osr)\ntransfer duration in minutes for the foot profile\n",
    ///      "type": "number"
    ///    },
    ///    "footRouted": {
    ///      "description": "optional; missing if no path was found with foot
    /// routing\ntransfer duration in minutes for the foot profile\n",
    ///      "type": "number"
    ///    },
    ///    "to": {
    ///      "$ref": "#/components/schemas/Place"
    ///    },
    ///    "wheelchair": {
    ///      "description": "optional; missing if no path was found with the
    /// wheelchair profile \ntransfer duration in minutes for the wheelchair
    /// profile\n",
    ///      "type": "number"
    ///    },
    ///    "wheelchairRouted": {
    ///      "description": "optional; missing if no path was found with the
    /// wheelchair profile\ntransfer duration in minutes for the wheelchair
    /// profile\n",
    ///      "type": "number"
    ///    },
    ///    "wheelchairUsesElevator": {
    ///      "description": "optional; missing if no path was found with the
    /// wheelchair profile\ntrue if the wheelchair path uses an elevator\n",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct Transfer {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub car: ::std::option::Option<f64>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub default: ::std::option::Option<f64>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub foot: ::std::option::Option<f64>,
        #[serde(
            rename = "footRouted",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub foot_routed: ::std::option::Option<f64>,
        pub to: Place,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub wheelchair: ::std::option::Option<f64>,
        #[serde(
            rename = "wheelchairRouted",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub wheelchair_routed: ::std::option::Option<f64>,
        ///optional; missing if no path was found with the wheelchair profile
        ///true if the wheelchair path uses an elevator
        #[serde(
            rename = "wheelchairUsesElevator",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub wheelchair_uses_elevator: ::std::option::Option<bool>,
    }

    impl ::std::convert::From<&Transfer> for Transfer {
        fn from(value: &Transfer) -> Self {
            value.clone()
        }
    }

    impl Transfer {
        pub fn builder() -> builder::Transfer {
            Default::default()
        }
    }

    ///`TransfersResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "hasCarTransfers",
    ///    "hasFootTransfers",
    ///    "hasWheelchairTransfers",
    ///    "place",
    ///    "transfers"
    ///  ],
    ///  "properties": {
    ///    "hasCarTransfers": {
    ///      "description": "true if the server has car transfers computed",
    ///      "type": "boolean"
    ///    },
    ///    "hasFootTransfers": {
    ///      "description": "true if the server has foot transfers computed",
    ///      "type": "boolean"
    ///    },
    ///    "hasWheelchairTransfers": {
    ///      "description": "true if the server has wheelchair transfers
    /// computed",
    ///      "type": "boolean"
    ///    },
    ///    "place": {
    ///      "$ref": "#/components/schemas/Place"
    ///    },
    ///    "transfers": {
    ///      "description": "all outgoing transfers of this location",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Transfer"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct TransfersResponse {
        ///true if the server has car transfers computed
        #[serde(rename = "hasCarTransfers")]
        pub has_car_transfers: bool,
        ///true if the server has foot transfers computed
        #[serde(rename = "hasFootTransfers")]
        pub has_foot_transfers: bool,
        ///true if the server has wheelchair transfers computed
        #[serde(rename = "hasWheelchairTransfers")]
        pub has_wheelchair_transfers: bool,
        pub place: Place,
        ///all outgoing transfers of this location
        pub transfers: ::std::vec::Vec<Transfer>,
    }

    impl ::std::convert::From<&TransfersResponse> for TransfersResponse {
        fn from(value: &TransfersResponse) -> Self {
            value.clone()
        }
    }

    impl TransfersResponse {
        pub fn builder() -> builder::TransfersResponse {
            Default::default()
        }
    }

    ///trip id and name
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "trip id and name",
    ///  "type": "object",
    ///  "required": [
    ///    "tripId"
    ///  ],
    ///  "properties": {
    ///    "displayName": {
    ///      "description": "trip display name (api version >= 4)",
    ///      "type": "string"
    ///    },
    ///    "routeShortName": {
    ///      "description": "trip display name (api version < 4)",
    ///      "type": "string"
    ///    },
    ///    "tripId": {
    ///      "description": "trip ID (dataset trip id prefixed with the dataset
    /// tag)",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct TripInfo {
        ///trip display name (api version >= 4)
        #[serde(
            rename = "displayName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub display_name: ::std::option::Option<::std::string::String>,
        ///trip display name (api version < 4)
        #[serde(
            rename = "routeShortName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_short_name: ::std::option::Option<::std::string::String>,
        ///trip ID (dataset trip id prefixed with the dataset tag)
        #[serde(rename = "tripId")]
        pub trip_id: ::std::string::String,
    }

    impl ::std::convert::From<&TripInfo> for TripInfo {
        fn from(value: &TripInfo) -> Self {
            value.clone()
        }
    }

    impl TripInfo {
        pub fn builder() -> builder::TripInfo {
            Default::default()
        }
    }

    ///trip segment between two stops to show a trip on a map
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "trip segment between two stops to show a trip on a
    /// map",
    ///  "type": "object",
    ///  "required": [
    ///    "arrival",
    ///    "departure",
    ///    "distance",
    ///    "from",
    ///    "mode",
    ///    "polyline",
    ///    "realTime",
    ///    "scheduledArrival",
    ///    "scheduledDeparture",
    ///    "to",
    ///    "trips"
    ///  ],
    ///  "properties": {
    ///    "arrival": {
    ///      "description": "arrival time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "departure": {
    ///      "description": "departure time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "distance": {
    ///      "description": "distance in meters",
    ///      "type": "number"
    ///    },
    ///    "from": {
    ///      "$ref": "#/components/schemas/Place"
    ///    },
    ///    "mode": {
    ///      "$ref": "#/components/schemas/Mode"
    ///    },
    ///    "polyline": {
    ///      "description": "Google polyline encoded coordinate sequence (with
    /// precision 5) where the trip travels on this segment.",
    ///      "type": "string"
    ///    },
    ///    "realTime": {
    ///      "description": "Whether there is real-time data about this leg",
    ///      "type": "boolean"
    ///    },
    ///    "routeColor": {
    ///      "type": "string"
    ///    },
    ///    "scheduledArrival": {
    ///      "description": "scheduled arrival time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "scheduledDeparture": {
    ///      "description": "scheduled departure time",
    ///      "type": "string",
    ///      "format": "date-time"
    ///    },
    ///    "to": {
    ///      "$ref": "#/components/schemas/Place"
    ///    },
    ///    "trips": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/TripInfo"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct TripSegment {
        ///arrival time
        pub arrival: ::chrono::DateTime<::chrono::offset::Utc>,
        ///departure time
        pub departure: ::chrono::DateTime<::chrono::offset::Utc>,
        pub distance: f64,
        pub from: Place,
        pub mode: Mode,
        ///Google polyline encoded coordinate sequence (with precision 5) where
        /// the trip travels on this segment.
        pub polyline: ::std::string::String,
        ///Whether there is real-time data about this leg
        #[serde(rename = "realTime")]
        pub real_time: bool,
        #[serde(
            rename = "routeColor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub route_color: ::std::option::Option<::std::string::String>,
        ///scheduled arrival time
        #[serde(rename = "scheduledArrival")]
        pub scheduled_arrival: ::chrono::DateTime<::chrono::offset::Utc>,
        ///scheduled departure time
        #[serde(rename = "scheduledDeparture")]
        pub scheduled_departure: ::chrono::DateTime<::chrono::offset::Utc>,
        pub to: Place,
        pub trips: ::std::vec::Vec<TripInfo>,
    }

    impl ::std::convert::From<&TripSegment> for TripSegment {
        fn from(value: &TripSegment) -> Self {
            value.clone()
        }
    }

    impl TripSegment {
        pub fn builder() -> builder::TripSegment {
            Default::default()
        }
    }

    /// - `NORMAL` - latitude / longitude coordinate or address
    /// - `BIKESHARE` - bike sharing station
    /// - `TRANSIT` - transit stop
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "- `NORMAL` - latitude / longitude coordinate or
    /// address\n- `BIKESHARE` - bike sharing station\n- `TRANSIT` - transit
    /// stop\n",
    ///  "type": "string",
    ///  "enum": [
    ///    "NORMAL",
    ///    "BIKESHARE",
    ///    "TRANSIT"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum VertexType {
        #[serde(rename = "NORMAL")]
        Normal,
        #[serde(rename = "BIKESHARE")]
        Bikeshare,
        #[serde(rename = "TRANSIT")]
        Transit,
    }

    impl ::std::convert::From<&Self> for VertexType {
        fn from(value: &VertexType) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for VertexType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Normal => f.write_str("NORMAL"),
                Self::Bikeshare => f.write_str("BIKESHARE"),
                Self::Transit => f.write_str("TRANSIT"),
            }
        }
    }

    impl ::std::str::FromStr for VertexType {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "NORMAL" => Ok(Self::Normal),
                "BIKESHARE" => Ok(Self::Bikeshare),
                "TRANSIT" => Ok(Self::Transit),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for VertexType {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for VertexType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for VertexType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    /// Types for composing complex structures.
    pub mod builder {
        #[derive(Clone, Debug)]
        pub struct Alert {
            cause: ::std::result::Result<
                ::std::option::Option<super::AlertCause>,
                ::std::string::String,
            >,
            cause_detail: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            communication_period:
                ::std::result::Result<::std::vec::Vec<super::TimeRange>, ::std::string::String>,
            description_text: ::std::result::Result<::std::string::String, ::std::string::String>,
            effect: ::std::result::Result<
                ::std::option::Option<super::AlertEffect>,
                ::std::string::String,
            >,
            effect_detail: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            header_text: ::std::result::Result<::std::string::String, ::std::string::String>,
            image_alternative_text: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            image_media_type: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            image_url: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            impact_period:
                ::std::result::Result<::std::vec::Vec<super::TimeRange>, ::std::string::String>,
            severity_level: ::std::result::Result<
                ::std::option::Option<super::AlertSeverityLevel>,
                ::std::string::String,
            >,
            tts_description_text: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            tts_header_text: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            url: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for Alert {
            fn default() -> Self {
                Self {
                    cause: Ok(Default::default()),
                    cause_detail: Ok(Default::default()),
                    communication_period: Ok(Default::default()),
                    description_text: Err("no value supplied for description_text".to_string()),
                    effect: Ok(Default::default()),
                    effect_detail: Ok(Default::default()),
                    header_text: Err("no value supplied for header_text".to_string()),
                    image_alternative_text: Ok(Default::default()),
                    image_media_type: Ok(Default::default()),
                    image_url: Ok(Default::default()),
                    impact_period: Ok(Default::default()),
                    severity_level: Ok(Default::default()),
                    tts_description_text: Ok(Default::default()),
                    tts_header_text: Ok(Default::default()),
                    url: Ok(Default::default()),
                }
            }
        }

        impl Alert {
            pub fn cause<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::AlertCause>>,
                T::Error: ::std::fmt::Display,
            {
                self.cause = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for cause: {}", e));
                self
            }
            pub fn cause_detail<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.cause_detail = value.try_into().map_err(|e| {
                    format!("error converting supplied value for cause_detail: {}", e)
                });
                self
            }
            pub fn communication_period<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::TimeRange>>,
                T::Error: ::std::fmt::Display,
            {
                self.communication_period = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for communication_period: {}",
                        e
                    )
                });
                self
            }
            pub fn description_text<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.description_text = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for description_text: {}",
                        e
                    )
                });
                self
            }
            pub fn effect<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::AlertEffect>>,
                T::Error: ::std::fmt::Display,
            {
                self.effect = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for effect: {}", e));
                self
            }
            pub fn effect_detail<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.effect_detail = value.try_into().map_err(|e| {
                    format!("error converting supplied value for effect_detail: {}", e)
                });
                self
            }
            pub fn header_text<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.header_text = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for header_text: {}", e));
                self
            }
            pub fn image_alternative_text<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.image_alternative_text = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for image_alternative_text: {}",
                        e
                    )
                });
                self
            }
            pub fn image_media_type<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.image_media_type = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for image_media_type: {}",
                        e
                    )
                });
                self
            }
            pub fn image_url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.image_url = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for image_url: {}", e));
                self
            }
            pub fn impact_period<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::TimeRange>>,
                T::Error: ::std::fmt::Display,
            {
                self.impact_period = value.try_into().map_err(|e| {
                    format!("error converting supplied value for impact_period: {}", e)
                });
                self
            }
            pub fn severity_level<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::AlertSeverityLevel>>,
                T::Error: ::std::fmt::Display,
            {
                self.severity_level = value.try_into().map_err(|e| {
                    format!("error converting supplied value for severity_level: {}", e)
                });
                self
            }
            pub fn tts_description_text<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.tts_description_text = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for tts_description_text: {}",
                        e
                    )
                });
                self
            }
            pub fn tts_header_text<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.tts_header_text = value.try_into().map_err(|e| {
                    format!("error converting supplied value for tts_header_text: {}", e)
                });
                self
            }
            pub fn url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.url = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for url: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<Alert> for super::Alert {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Alert,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    cause: value.cause?,
                    cause_detail: value.cause_detail?,
                    communication_period: value.communication_period?,
                    description_text: value.description_text?,
                    effect: value.effect?,
                    effect_detail: value.effect_detail?,
                    header_text: value.header_text?,
                    image_alternative_text: value.image_alternative_text?,
                    image_media_type: value.image_media_type?,
                    image_url: value.image_url?,
                    impact_period: value.impact_period?,
                    severity_level: value.severity_level?,
                    tts_description_text: value.tts_description_text?,
                    tts_header_text: value.tts_header_text?,
                    url: value.url?,
                })
            }
        }

        impl ::std::convert::From<super::Alert> for Alert {
            fn from(value: super::Alert) -> Self {
                Self {
                    cause: Ok(value.cause),
                    cause_detail: Ok(value.cause_detail),
                    communication_period: Ok(value.communication_period),
                    description_text: Ok(value.description_text),
                    effect: Ok(value.effect),
                    effect_detail: Ok(value.effect_detail),
                    header_text: Ok(value.header_text),
                    image_alternative_text: Ok(value.image_alternative_text),
                    image_media_type: Ok(value.image_media_type),
                    image_url: Ok(value.image_url),
                    impact_period: Ok(value.impact_period),
                    severity_level: Ok(value.severity_level),
                    tts_description_text: Ok(value.tts_description_text),
                    tts_header_text: Ok(value.tts_header_text),
                    url: Ok(value.url),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Area {
            admin_level: ::std::result::Result<f64, ::std::string::String>,
            default: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
            matched: ::std::result::Result<bool, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            unique: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        }

        impl ::std::default::Default for Area {
            fn default() -> Self {
                Self {
                    admin_level: Err("no value supplied for admin_level".to_string()),
                    default: Ok(Default::default()),
                    matched: Err("no value supplied for matched".to_string()),
                    name: Err("no value supplied for name".to_string()),
                    unique: Ok(Default::default()),
                }
            }
        }

        impl Area {
            pub fn admin_level<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.admin_level = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for admin_level: {}", e));
                self
            }
            pub fn default<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<bool>>,
                T::Error: ::std::fmt::Display,
            {
                self.default = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for default: {}", e));
                self
            }
            pub fn matched<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.matched = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for matched: {}", e));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {}", e));
                self
            }
            pub fn unique<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<bool>>,
                T::Error: ::std::fmt::Display,
            {
                self.unique = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for unique: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<Area> for super::Area {
            type Error = super::error::ConversionError;
            fn try_from(value: Area) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    admin_level: value.admin_level?,
                    default: value.default?,
                    matched: value.matched?,
                    name: value.name?,
                    unique: value.unique?,
                })
            }
        }

        impl ::std::convert::From<super::Area> for Area {
            fn from(value: super::Area) -> Self {
                Self {
                    admin_level: Ok(value.admin_level),
                    default: Ok(value.default),
                    matched: Ok(value.matched),
                    name: Ok(value.name),
                    unique: Ok(value.unique),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Duration {
            duration: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        }

        impl ::std::default::Default for Duration {
            fn default() -> Self {
                Self {
                    duration: Ok(Default::default()),
                }
            }
        }

        impl Duration {
            pub fn duration<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<f64>>,
                T::Error: ::std::fmt::Display,
            {
                self.duration = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for duration: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<Duration> for super::Duration {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Duration,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    duration: value.duration?,
                })
            }
        }

        impl ::std::convert::From<super::Duration> for Duration {
            fn from(value: super::Duration) -> Self {
                Self {
                    duration: Ok(value.duration),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct EncodedPolyline {
            length: ::std::result::Result<u64, ::std::string::String>,
            points: ::std::result::Result<::std::string::String, ::std::string::String>,
            precision: ::std::result::Result<i64, ::std::string::String>,
        }

        impl ::std::default::Default for EncodedPolyline {
            fn default() -> Self {
                Self {
                    length: Err("no value supplied for length".to_string()),
                    points: Err("no value supplied for points".to_string()),
                    precision: Err("no value supplied for precision".to_string()),
                }
            }
        }

        impl EncodedPolyline {
            pub fn length<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u64>,
                T::Error: ::std::fmt::Display,
            {
                self.length = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for length: {}", e));
                self
            }
            pub fn points<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.points = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for points: {}", e));
                self
            }
            pub fn precision<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i64>,
                T::Error: ::std::fmt::Display,
            {
                self.precision = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for precision: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<EncodedPolyline> for super::EncodedPolyline {
            type Error = super::error::ConversionError;
            fn try_from(
                value: EncodedPolyline,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    length: value.length?,
                    points: value.points?,
                    precision: value.precision?,
                })
            }
        }

        impl ::std::convert::From<super::EncodedPolyline> for EncodedPolyline {
            fn from(value: super::EncodedPolyline) -> Self {
                Self {
                    length: Ok(value.length),
                    points: Ok(value.points),
                    precision: Ok(value.precision),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct FareMedia {
            fare_media_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            fare_media_type: ::std::result::Result<super::FareMediaType, ::std::string::String>,
        }

        impl ::std::default::Default for FareMedia {
            fn default() -> Self {
                Self {
                    fare_media_name: Ok(Default::default()),
                    fare_media_type: Err("no value supplied for fare_media_type".to_string()),
                }
            }
        }

        impl FareMedia {
            pub fn fare_media_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.fare_media_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for fare_media_name: {}", e)
                });
                self
            }
            pub fn fare_media_type<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::FareMediaType>,
                T::Error: ::std::fmt::Display,
            {
                self.fare_media_type = value.try_into().map_err(|e| {
                    format!("error converting supplied value for fare_media_type: {}", e)
                });
                self
            }
        }

        impl ::std::convert::TryFrom<FareMedia> for super::FareMedia {
            type Error = super::error::ConversionError;
            fn try_from(
                value: FareMedia,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    fare_media_name: value.fare_media_name?,
                    fare_media_type: value.fare_media_type?,
                })
            }
        }

        impl ::std::convert::From<super::FareMedia> for FareMedia {
            fn from(value: super::FareMedia) -> Self {
                Self {
                    fare_media_name: Ok(value.fare_media_name),
                    fare_media_type: Ok(value.fare_media_type),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct FareProduct {
            amount: ::std::result::Result<f64, ::std::string::String>,
            currency: ::std::result::Result<::std::string::String, ::std::string::String>,
            media: ::std::result::Result<
                ::std::option::Option<super::FareMedia>,
                ::std::string::String,
            >,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            rider_category: ::std::result::Result<
                ::std::option::Option<super::RiderCategory>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for FareProduct {
            fn default() -> Self {
                Self {
                    amount: Err("no value supplied for amount".to_string()),
                    currency: Err("no value supplied for currency".to_string()),
                    media: Ok(Default::default()),
                    name: Err("no value supplied for name".to_string()),
                    rider_category: Ok(Default::default()),
                }
            }
        }

        impl FareProduct {
            pub fn amount<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.amount = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for amount: {}", e));
                self
            }
            pub fn currency<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.currency = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for currency: {}", e));
                self
            }
            pub fn media<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::FareMedia>>,
                T::Error: ::std::fmt::Display,
            {
                self.media = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for media: {}", e));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {}", e));
                self
            }
            pub fn rider_category<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::RiderCategory>>,
                T::Error: ::std::fmt::Display,
            {
                self.rider_category = value.try_into().map_err(|e| {
                    format!("error converting supplied value for rider_category: {}", e)
                });
                self
            }
        }

        impl ::std::convert::TryFrom<FareProduct> for super::FareProduct {
            type Error = super::error::ConversionError;
            fn try_from(
                value: FareProduct,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    amount: value.amount?,
                    currency: value.currency?,
                    media: value.media?,
                    name: value.name?,
                    rider_category: value.rider_category?,
                })
            }
        }

        impl ::std::convert::From<super::FareProduct> for FareProduct {
            fn from(value: super::FareProduct) -> Self {
                Self {
                    amount: Ok(value.amount),
                    currency: Ok(value.currency),
                    media: Ok(value.media),
                    name: Ok(value.name),
                    rider_category: Ok(value.rider_category),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct FareTransfer {
            effective_fare_leg_products: ::std::result::Result<
                ::std::vec::Vec<::std::vec::Vec<::std::vec::Vec<super::FareProduct>>>,
                ::std::string::String,
            >,
            rule: ::std::result::Result<
                ::std::option::Option<super::FareTransferRule>,
                ::std::string::String,
            >,
            transfer_products:
                ::std::result::Result<::std::vec::Vec<super::FareProduct>, ::std::string::String>,
        }

        impl ::std::default::Default for FareTransfer {
            fn default() -> Self {
                Self {
                    effective_fare_leg_products: Err(
                        "no value supplied for effective_fare_leg_products".to_string(),
                    ),
                    rule: Ok(Default::default()),
                    transfer_products: Ok(Default::default()),
                }
            }
        }

        impl FareTransfer {
            pub fn effective_fare_leg_products<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::vec::Vec<::std::vec::Vec<::std::vec::Vec<super::FareProduct>>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.effective_fare_leg_products = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for effective_fare_leg_products: {}",
                        e
                    )
                });
                self
            }
            pub fn rule<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::FareTransferRule>>,
                T::Error: ::std::fmt::Display,
            {
                self.rule = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for rule: {}", e));
                self
            }
            pub fn transfer_products<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::FareProduct>>,
                T::Error: ::std::fmt::Display,
            {
                self.transfer_products = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for transfer_products: {}",
                        e
                    )
                });
                self
            }
        }

        impl ::std::convert::TryFrom<FareTransfer> for super::FareTransfer {
            type Error = super::error::ConversionError;
            fn try_from(
                value: FareTransfer,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    effective_fare_leg_products: value.effective_fare_leg_products?,
                    rule: value.rule?,
                    transfer_products: value.transfer_products?,
                })
            }
        }

        impl ::std::convert::From<super::FareTransfer> for FareTransfer {
            fn from(value: super::FareTransfer) -> Self {
                Self {
                    effective_fare_leg_products: Ok(value.effective_fare_leg_products),
                    rule: Ok(value.rule),
                    transfer_products: Ok(value.transfer_products),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InitialResponse {
            lat: ::std::result::Result<f64, ::std::string::String>,
            lon: ::std::result::Result<f64, ::std::string::String>,
            zoom: ::std::result::Result<f64, ::std::string::String>,
        }

        impl ::std::default::Default for InitialResponse {
            fn default() -> Self {
                Self {
                    lat: Err("no value supplied for lat".to_string()),
                    lon: Err("no value supplied for lon".to_string()),
                    zoom: Err("no value supplied for zoom".to_string()),
                }
            }
        }

        impl InitialResponse {
            pub fn lat<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.lat = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for lat: {}", e));
                self
            }
            pub fn lon<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.lon = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for lon: {}", e));
                self
            }
            pub fn zoom<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.zoom = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for zoom: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<InitialResponse> for super::InitialResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InitialResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    lat: value.lat?,
                    lon: value.lon?,
                    zoom: value.zoom?,
                })
            }
        }

        impl ::std::convert::From<super::InitialResponse> for InitialResponse {
            fn from(value: super::InitialResponse) -> Self {
                Self {
                    lat: Ok(value.lat),
                    lon: Ok(value.lon),
                    zoom: Ok(value.zoom),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Itinerary {
            duration: ::std::result::Result<i64, ::std::string::String>,
            end_time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            fare_transfers:
                ::std::result::Result<::std::vec::Vec<super::FareTransfer>, ::std::string::String>,
            legs: ::std::result::Result<::std::vec::Vec<super::Leg>, ::std::string::String>,
            start_time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            transfers: ::std::result::Result<i64, ::std::string::String>,
        }

        impl ::std::default::Default for Itinerary {
            fn default() -> Self {
                Self {
                    duration: Err("no value supplied for duration".to_string()),
                    end_time: Err("no value supplied for end_time".to_string()),
                    fare_transfers: Ok(Default::default()),
                    legs: Err("no value supplied for legs".to_string()),
                    start_time: Err("no value supplied for start_time".to_string()),
                    transfers: Err("no value supplied for transfers".to_string()),
                }
            }
        }

        impl Itinerary {
            pub fn duration<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i64>,
                T::Error: ::std::fmt::Display,
            {
                self.duration = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for duration: {}", e));
                self
            }
            pub fn end_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.end_time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for end_time: {}", e));
                self
            }
            pub fn fare_transfers<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::FareTransfer>>,
                T::Error: ::std::fmt::Display,
            {
                self.fare_transfers = value.try_into().map_err(|e| {
                    format!("error converting supplied value for fare_transfers: {}", e)
                });
                self
            }
            pub fn legs<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Leg>>,
                T::Error: ::std::fmt::Display,
            {
                self.legs = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for legs: {}", e));
                self
            }
            pub fn start_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.start_time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for start_time: {}", e));
                self
            }
            pub fn transfers<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i64>,
                T::Error: ::std::fmt::Display,
            {
                self.transfers = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for transfers: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<Itinerary> for super::Itinerary {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Itinerary,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    duration: value.duration?,
                    end_time: value.end_time?,
                    fare_transfers: value.fare_transfers?,
                    legs: value.legs?,
                    start_time: value.start_time?,
                    transfers: value.transfers?,
                })
            }
        }

        impl ::std::convert::From<super::Itinerary> for Itinerary {
            fn from(value: super::Itinerary) -> Self {
                Self {
                    duration: Ok(value.duration),
                    end_time: Ok(value.end_time),
                    fare_transfers: Ok(value.fare_transfers),
                    legs: Ok(value.legs),
                    start_time: Ok(value.start_time),
                    transfers: Ok(value.transfers),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Leg {
            agency_id: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            agency_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            agency_url: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            alerts: ::std::result::Result<::std::vec::Vec<super::Alert>, ::std::string::String>,
            cancelled: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
            display_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            distance: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
            duration: ::std::result::Result<i64, ::std::string::String>,
            effective_fare_leg_index:
                ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
            end_time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            fare_transfer_index:
                ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
            from: ::std::result::Result<super::Place, ::std::string::String>,
            headsign: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            interline_with_previous_leg:
                ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
            intermediate_stops:
                ::std::result::Result<::std::vec::Vec<super::Place>, ::std::string::String>,
            leg_geometry: ::std::result::Result<super::EncodedPolyline, ::std::string::String>,
            looped_calendar_since: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
            mode: ::std::result::Result<super::Mode, ::std::string::String>,
            real_time: ::std::result::Result<bool, ::std::string::String>,
            rental:
                ::std::result::Result<::std::option::Option<super::Rental>, ::std::string::String>,
            route_color: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            route_long_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            route_short_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            route_text_color: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            route_type: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
            scheduled: ::std::result::Result<bool, ::std::string::String>,
            scheduled_end_time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            scheduled_start_time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            source: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            start_time: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            steps: ::std::result::Result<
                ::std::vec::Vec<super::StepInstruction>,
                ::std::string::String,
            >,
            to: ::std::result::Result<super::Place, ::std::string::String>,
            trip_id: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            trip_short_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            trip_to:
                ::std::result::Result<::std::option::Option<super::Place>, ::std::string::String>,
        }

        impl ::std::default::Default for Leg {
            fn default() -> Self {
                Self {
                    agency_id: Ok(Default::default()),
                    agency_name: Ok(Default::default()),
                    agency_url: Ok(Default::default()),
                    alerts: Ok(Default::default()),
                    cancelled: Ok(Default::default()),
                    display_name: Ok(Default::default()),
                    distance: Ok(Default::default()),
                    duration: Err("no value supplied for duration".to_string()),
                    effective_fare_leg_index: Ok(Default::default()),
                    end_time: Err("no value supplied for end_time".to_string()),
                    fare_transfer_index: Ok(Default::default()),
                    from: Err("no value supplied for from".to_string()),
                    headsign: Ok(Default::default()),
                    interline_with_previous_leg: Ok(Default::default()),
                    intermediate_stops: Ok(Default::default()),
                    leg_geometry: Err("no value supplied for leg_geometry".to_string()),
                    looped_calendar_since: Ok(Default::default()),
                    mode: Err("no value supplied for mode".to_string()),
                    real_time: Err("no value supplied for real_time".to_string()),
                    rental: Ok(Default::default()),
                    route_color: Ok(Default::default()),
                    route_long_name: Ok(Default::default()),
                    route_short_name: Ok(Default::default()),
                    route_text_color: Ok(Default::default()),
                    route_type: Ok(Default::default()),
                    scheduled: Err("no value supplied for scheduled".to_string()),
                    scheduled_end_time: Err("no value supplied for scheduled_end_time".to_string()),
                    scheduled_start_time: Err(
                        "no value supplied for scheduled_start_time".to_string()
                    ),
                    source: Ok(Default::default()),
                    start_time: Err("no value supplied for start_time".to_string()),
                    steps: Ok(Default::default()),
                    to: Err("no value supplied for to".to_string()),
                    trip_id: Ok(Default::default()),
                    trip_short_name: Ok(Default::default()),
                    trip_to: Ok(Default::default()),
                }
            }
        }

        impl Leg {
            pub fn agency_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.agency_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for agency_id: {}", e));
                self
            }
            pub fn agency_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.agency_name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for agency_name: {}", e));
                self
            }
            pub fn agency_url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.agency_url = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for agency_url: {}", e));
                self
            }
            pub fn alerts<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Alert>>,
                T::Error: ::std::fmt::Display,
            {
                self.alerts = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for alerts: {}", e));
                self
            }
            pub fn cancelled<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<bool>>,
                T::Error: ::std::fmt::Display,
            {
                self.cancelled = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for cancelled: {}", e));
                self
            }
            pub fn display_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.display_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for display_name: {}", e)
                });
                self
            }
            pub fn distance<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<f64>>,
                T::Error: ::std::fmt::Display,
            {
                self.distance = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for distance: {}", e));
                self
            }
            pub fn duration<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i64>,
                T::Error: ::std::fmt::Display,
            {
                self.duration = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for duration: {}", e));
                self
            }
            pub fn effective_fare_leg_index<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.effective_fare_leg_index = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for effective_fare_leg_index: {}",
                        e
                    )
                });
                self
            }
            pub fn end_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.end_time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for end_time: {}", e));
                self
            }
            pub fn fare_transfer_index<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.fare_transfer_index = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for fare_transfer_index: {}",
                        e
                    )
                });
                self
            }
            pub fn from<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.from = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for from: {}", e));
                self
            }
            pub fn headsign<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.headsign = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for headsign: {}", e));
                self
            }
            pub fn interline_with_previous_leg<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<bool>>,
                T::Error: ::std::fmt::Display,
            {
                self.interline_with_previous_leg = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for interline_with_previous_leg: {}",
                        e
                    )
                });
                self
            }
            pub fn intermediate_stops<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Place>>,
                T::Error: ::std::fmt::Display,
            {
                self.intermediate_stops = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for intermediate_stops: {}",
                        e
                    )
                });
                self
            }
            pub fn leg_geometry<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::EncodedPolyline>,
                T::Error: ::std::fmt::Display,
            {
                self.leg_geometry = value.try_into().map_err(|e| {
                    format!("error converting supplied value for leg_geometry: {}", e)
                });
                self
            }
            pub fn looped_calendar_since<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.looped_calendar_since = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for looped_calendar_since: {}",
                        e
                    )
                });
                self
            }
            pub fn mode<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Mode>,
                T::Error: ::std::fmt::Display,
            {
                self.mode = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for mode: {}", e));
                self
            }
            pub fn real_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.real_time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for real_time: {}", e));
                self
            }
            pub fn rental<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::Rental>>,
                T::Error: ::std::fmt::Display,
            {
                self.rental = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for rental: {}", e));
                self
            }
            pub fn route_color<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_color = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for route_color: {}", e));
                self
            }
            pub fn route_long_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_long_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for route_long_name: {}", e)
                });
                self
            }
            pub fn route_short_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_short_name = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for route_short_name: {}",
                        e
                    )
                });
                self
            }
            pub fn route_text_color<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_text_color = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for route_text_color: {}",
                        e
                    )
                });
                self
            }
            pub fn route_type<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_type = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for route_type: {}", e));
                self
            }
            pub fn scheduled<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.scheduled = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for scheduled: {}", e));
                self
            }
            pub fn scheduled_end_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.scheduled_end_time = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for scheduled_end_time: {}",
                        e
                    )
                });
                self
            }
            pub fn scheduled_start_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.scheduled_start_time = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for scheduled_start_time: {}",
                        e
                    )
                });
                self
            }
            pub fn source<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.source = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for source: {}", e));
                self
            }
            pub fn start_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.start_time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for start_time: {}", e));
                self
            }
            pub fn steps<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::StepInstruction>>,
                T::Error: ::std::fmt::Display,
            {
                self.steps = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for steps: {}", e));
                self
            }
            pub fn to<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.to = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for to: {}", e));
                self
            }
            pub fn trip_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.trip_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for trip_id: {}", e));
                self
            }
            pub fn trip_short_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.trip_short_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for trip_short_name: {}", e)
                });
                self
            }
            pub fn trip_to<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::Place>>,
                T::Error: ::std::fmt::Display,
            {
                self.trip_to = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for trip_to: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<Leg> for super::Leg {
            type Error = super::error::ConversionError;
            fn try_from(value: Leg) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    agency_id: value.agency_id?,
                    agency_name: value.agency_name?,
                    agency_url: value.agency_url?,
                    alerts: value.alerts?,
                    cancelled: value.cancelled?,
                    display_name: value.display_name?,
                    distance: value.distance?,
                    duration: value.duration?,
                    effective_fare_leg_index: value.effective_fare_leg_index?,
                    end_time: value.end_time?,
                    fare_transfer_index: value.fare_transfer_index?,
                    from: value.from?,
                    headsign: value.headsign?,
                    interline_with_previous_leg: value.interline_with_previous_leg?,
                    intermediate_stops: value.intermediate_stops?,
                    leg_geometry: value.leg_geometry?,
                    looped_calendar_since: value.looped_calendar_since?,
                    mode: value.mode?,
                    real_time: value.real_time?,
                    rental: value.rental?,
                    route_color: value.route_color?,
                    route_long_name: value.route_long_name?,
                    route_short_name: value.route_short_name?,
                    route_text_color: value.route_text_color?,
                    route_type: value.route_type?,
                    scheduled: value.scheduled?,
                    scheduled_end_time: value.scheduled_end_time?,
                    scheduled_start_time: value.scheduled_start_time?,
                    source: value.source?,
                    start_time: value.start_time?,
                    steps: value.steps?,
                    to: value.to?,
                    trip_id: value.trip_id?,
                    trip_short_name: value.trip_short_name?,
                    trip_to: value.trip_to?,
                })
            }
        }

        impl ::std::convert::From<super::Leg> for Leg {
            fn from(value: super::Leg) -> Self {
                Self {
                    agency_id: Ok(value.agency_id),
                    agency_name: Ok(value.agency_name),
                    agency_url: Ok(value.agency_url),
                    alerts: Ok(value.alerts),
                    cancelled: Ok(value.cancelled),
                    display_name: Ok(value.display_name),
                    distance: Ok(value.distance),
                    duration: Ok(value.duration),
                    effective_fare_leg_index: Ok(value.effective_fare_leg_index),
                    end_time: Ok(value.end_time),
                    fare_transfer_index: Ok(value.fare_transfer_index),
                    from: Ok(value.from),
                    headsign: Ok(value.headsign),
                    interline_with_previous_leg: Ok(value.interline_with_previous_leg),
                    intermediate_stops: Ok(value.intermediate_stops),
                    leg_geometry: Ok(value.leg_geometry),
                    looped_calendar_since: Ok(value.looped_calendar_since),
                    mode: Ok(value.mode),
                    real_time: Ok(value.real_time),
                    rental: Ok(value.rental),
                    route_color: Ok(value.route_color),
                    route_long_name: Ok(value.route_long_name),
                    route_short_name: Ok(value.route_short_name),
                    route_text_color: Ok(value.route_text_color),
                    route_type: Ok(value.route_type),
                    scheduled: Ok(value.scheduled),
                    scheduled_end_time: Ok(value.scheduled_end_time),
                    scheduled_start_time: Ok(value.scheduled_start_time),
                    source: Ok(value.source),
                    start_time: Ok(value.start_time),
                    steps: Ok(value.steps),
                    to: Ok(value.to),
                    trip_id: Ok(value.trip_id),
                    trip_short_name: Ok(value.trip_short_name),
                    trip_to: Ok(value.trip_to),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Match {
            areas: ::std::result::Result<::std::vec::Vec<super::Area>, ::std::string::String>,
            house_number: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            lat: ::std::result::Result<f64, ::std::string::String>,
            level: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
            lon: ::std::result::Result<f64, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            score: ::std::result::Result<f64, ::std::string::String>,
            street: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            tokens: ::std::result::Result<::std::vec::Vec<super::Token>, ::std::string::String>,
            type_: ::std::result::Result<super::LocationType, ::std::string::String>,
            tz: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            zip: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for Match {
            fn default() -> Self {
                Self {
                    areas: Err("no value supplied for areas".to_string()),
                    house_number: Ok(Default::default()),
                    id: Err("no value supplied for id".to_string()),
                    lat: Err("no value supplied for lat".to_string()),
                    level: Ok(Default::default()),
                    lon: Err("no value supplied for lon".to_string()),
                    name: Err("no value supplied for name".to_string()),
                    score: Err("no value supplied for score".to_string()),
                    street: Ok(Default::default()),
                    tokens: Err("no value supplied for tokens".to_string()),
                    type_: Err("no value supplied for type_".to_string()),
                    tz: Ok(Default::default()),
                    zip: Ok(Default::default()),
                }
            }
        }

        impl Match {
            pub fn areas<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Area>>,
                T::Error: ::std::fmt::Display,
            {
                self.areas = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for areas: {}", e));
                self
            }
            pub fn house_number<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.house_number = value.try_into().map_err(|e| {
                    format!("error converting supplied value for house_number: {}", e)
                });
                self
            }
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {}", e));
                self
            }
            pub fn lat<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.lat = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for lat: {}", e));
                self
            }
            pub fn level<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<f64>>,
                T::Error: ::std::fmt::Display,
            {
                self.level = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for level: {}", e));
                self
            }
            pub fn lon<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.lon = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for lon: {}", e));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {}", e));
                self
            }
            pub fn score<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.score = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for score: {}", e));
                self
            }
            pub fn street<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.street = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for street: {}", e));
                self
            }
            pub fn tokens<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Token>>,
                T::Error: ::std::fmt::Display,
            {
                self.tokens = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for tokens: {}", e));
                self
            }
            pub fn type_<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::LocationType>,
                T::Error: ::std::fmt::Display,
            {
                self.type_ = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for type_: {}", e));
                self
            }
            pub fn tz<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.tz = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for tz: {}", e));
                self
            }
            pub fn zip<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.zip = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for zip: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<Match> for super::Match {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Match,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    areas: value.areas?,
                    house_number: value.house_number?,
                    id: value.id?,
                    lat: value.lat?,
                    level: value.level?,
                    lon: value.lon?,
                    name: value.name?,
                    score: value.score?,
                    street: value.street?,
                    tokens: value.tokens?,
                    type_: value.type_?,
                    tz: value.tz?,
                    zip: value.zip?,
                })
            }
        }

        impl ::std::convert::From<super::Match> for Match {
            fn from(value: super::Match) -> Self {
                Self {
                    areas: Ok(value.areas),
                    house_number: Ok(value.house_number),
                    id: Ok(value.id),
                    lat: Ok(value.lat),
                    level: Ok(value.level),
                    lon: Ok(value.lon),
                    name: Ok(value.name),
                    score: Ok(value.score),
                    street: Ok(value.street),
                    tokens: Ok(value.tokens),
                    type_: Ok(value.type_),
                    tz: Ok(value.tz),
                    zip: Ok(value.zip),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Place {
            alerts: ::std::result::Result<::std::vec::Vec<super::Alert>, ::std::string::String>,
            arrival: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
            cancelled: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
            departure: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
            description: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            dropoff_type: ::std::result::Result<
                ::std::option::Option<super::PickupDropoffType>,
                ::std::string::String,
            >,
            flex: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            flex_end_pickup_drop_off_window: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
            flex_id: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            flex_start_pickup_drop_off_window: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
            lat: ::std::result::Result<f64, ::std::string::String>,
            level: ::std::result::Result<f64, ::std::string::String>,
            lon: ::std::result::Result<f64, ::std::string::String>,
            name: ::std::result::Result<::std::string::String, ::std::string::String>,
            pickup_type: ::std::result::Result<
                ::std::option::Option<super::PickupDropoffType>,
                ::std::string::String,
            >,
            scheduled_arrival: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
            scheduled_departure: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
            scheduled_track: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            stop_id: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            track: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            tz: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            vertex_type: ::std::result::Result<
                ::std::option::Option<super::VertexType>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for Place {
            fn default() -> Self {
                Self {
                    alerts: Ok(Default::default()),
                    arrival: Ok(Default::default()),
                    cancelled: Ok(Default::default()),
                    departure: Ok(Default::default()),
                    description: Ok(Default::default()),
                    dropoff_type: Ok(Default::default()),
                    flex: Ok(Default::default()),
                    flex_end_pickup_drop_off_window: Ok(Default::default()),
                    flex_id: Ok(Default::default()),
                    flex_start_pickup_drop_off_window: Ok(Default::default()),
                    lat: Err("no value supplied for lat".to_string()),
                    level: Err("no value supplied for level".to_string()),
                    lon: Err("no value supplied for lon".to_string()),
                    name: Err("no value supplied for name".to_string()),
                    pickup_type: Ok(Default::default()),
                    scheduled_arrival: Ok(Default::default()),
                    scheduled_departure: Ok(Default::default()),
                    scheduled_track: Ok(Default::default()),
                    stop_id: Ok(Default::default()),
                    track: Ok(Default::default()),
                    tz: Ok(Default::default()),
                    vertex_type: Ok(Default::default()),
                }
            }
        }

        impl Place {
            pub fn alerts<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Alert>>,
                T::Error: ::std::fmt::Display,
            {
                self.alerts = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for alerts: {}", e));
                self
            }
            pub fn arrival<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.arrival = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for arrival: {}", e));
                self
            }
            pub fn cancelled<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<bool>>,
                T::Error: ::std::fmt::Display,
            {
                self.cancelled = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for cancelled: {}", e));
                self
            }
            pub fn departure<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.departure = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for departure: {}", e));
                self
            }
            pub fn description<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.description = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for description: {}", e));
                self
            }
            pub fn dropoff_type<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::PickupDropoffType>>,
                T::Error: ::std::fmt::Display,
            {
                self.dropoff_type = value.try_into().map_err(|e| {
                    format!("error converting supplied value for dropoff_type: {}", e)
                });
                self
            }
            pub fn flex<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.flex = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for flex: {}", e));
                self
            }
            pub fn flex_end_pickup_drop_off_window<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.flex_end_pickup_drop_off_window = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for flex_end_pickup_drop_off_window: {}",
                        e
                    )
                });
                self
            }
            pub fn flex_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.flex_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for flex_id: {}", e));
                self
            }
            pub fn flex_start_pickup_drop_off_window<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.flex_start_pickup_drop_off_window = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for flex_start_pickup_drop_off_window: {}",
                        e
                    )
                });
                self
            }
            pub fn lat<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.lat = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for lat: {}", e));
                self
            }
            pub fn level<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.level = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for level: {}", e));
                self
            }
            pub fn lon<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.lon = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for lon: {}", e));
                self
            }
            pub fn name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for name: {}", e));
                self
            }
            pub fn pickup_type<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::PickupDropoffType>>,
                T::Error: ::std::fmt::Display,
            {
                self.pickup_type = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for pickup_type: {}", e));
                self
            }
            pub fn scheduled_arrival<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.scheduled_arrival = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for scheduled_arrival: {}",
                        e
                    )
                });
                self
            }
            pub fn scheduled_departure<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.scheduled_departure = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for scheduled_departure: {}",
                        e
                    )
                });
                self
            }
            pub fn scheduled_track<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.scheduled_track = value.try_into().map_err(|e| {
                    format!("error converting supplied value for scheduled_track: {}", e)
                });
                self
            }
            pub fn stop_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.stop_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for stop_id: {}", e));
                self
            }
            pub fn track<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.track = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for track: {}", e));
                self
            }
            pub fn tz<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.tz = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for tz: {}", e));
                self
            }
            pub fn vertex_type<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::VertexType>>,
                T::Error: ::std::fmt::Display,
            {
                self.vertex_type = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for vertex_type: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<Place> for super::Place {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Place,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    alerts: value.alerts?,
                    arrival: value.arrival?,
                    cancelled: value.cancelled?,
                    departure: value.departure?,
                    description: value.description?,
                    dropoff_type: value.dropoff_type?,
                    flex: value.flex?,
                    flex_end_pickup_drop_off_window: value.flex_end_pickup_drop_off_window?,
                    flex_id: value.flex_id?,
                    flex_start_pickup_drop_off_window: value.flex_start_pickup_drop_off_window?,
                    lat: value.lat?,
                    level: value.level?,
                    lon: value.lon?,
                    name: value.name?,
                    pickup_type: value.pickup_type?,
                    scheduled_arrival: value.scheduled_arrival?,
                    scheduled_departure: value.scheduled_departure?,
                    scheduled_track: value.scheduled_track?,
                    stop_id: value.stop_id?,
                    track: value.track?,
                    tz: value.tz?,
                    vertex_type: value.vertex_type?,
                })
            }
        }

        impl ::std::convert::From<super::Place> for Place {
            fn from(value: super::Place) -> Self {
                Self {
                    alerts: Ok(value.alerts),
                    arrival: Ok(value.arrival),
                    cancelled: Ok(value.cancelled),
                    departure: Ok(value.departure),
                    description: Ok(value.description),
                    dropoff_type: Ok(value.dropoff_type),
                    flex: Ok(value.flex),
                    flex_end_pickup_drop_off_window: Ok(value.flex_end_pickup_drop_off_window),
                    flex_id: Ok(value.flex_id),
                    flex_start_pickup_drop_off_window: Ok(value.flex_start_pickup_drop_off_window),
                    lat: Ok(value.lat),
                    level: Ok(value.level),
                    lon: Ok(value.lon),
                    name: Ok(value.name),
                    pickup_type: Ok(value.pickup_type),
                    scheduled_arrival: Ok(value.scheduled_arrival),
                    scheduled_departure: Ok(value.scheduled_departure),
                    scheduled_track: Ok(value.scheduled_track),
                    stop_id: Ok(value.stop_id),
                    track: Ok(value.track),
                    tz: Ok(value.tz),
                    vertex_type: Ok(value.vertex_type),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct PlanResponse {
            debug_output: ::std::result::Result<
                ::std::collections::HashMap<::std::string::String, i64>,
                ::std::string::String,
            >,
            direct: ::std::result::Result<::std::vec::Vec<super::Itinerary>, ::std::string::String>,
            from: ::std::result::Result<super::Place, ::std::string::String>,
            itineraries:
                ::std::result::Result<::std::vec::Vec<super::Itinerary>, ::std::string::String>,
            next_page_cursor: ::std::result::Result<::std::string::String, ::std::string::String>,
            previous_page_cursor:
                ::std::result::Result<::std::string::String, ::std::string::String>,
            request_parameters: ::std::result::Result<
                ::std::collections::HashMap<::std::string::String, ::std::string::String>,
                ::std::string::String,
            >,
            to: ::std::result::Result<super::Place, ::std::string::String>,
        }

        impl ::std::default::Default for PlanResponse {
            fn default() -> Self {
                Self {
                    debug_output: Err("no value supplied for debug_output".to_string()),
                    direct: Err("no value supplied for direct".to_string()),
                    from: Err("no value supplied for from".to_string()),
                    itineraries: Err("no value supplied for itineraries".to_string()),
                    next_page_cursor: Err("no value supplied for next_page_cursor".to_string()),
                    previous_page_cursor: Err(
                        "no value supplied for previous_page_cursor".to_string()
                    ),
                    request_parameters: Err("no value supplied for request_parameters".to_string()),
                    to: Err("no value supplied for to".to_string()),
                }
            }
        }

        impl PlanResponse {
            pub fn debug_output<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::collections::HashMap<::std::string::String, i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.debug_output = value.try_into().map_err(|e| {
                    format!("error converting supplied value for debug_output: {}", e)
                });
                self
            }
            pub fn direct<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Itinerary>>,
                T::Error: ::std::fmt::Display,
            {
                self.direct = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for direct: {}", e));
                self
            }
            pub fn from<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.from = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for from: {}", e));
                self
            }
            pub fn itineraries<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Itinerary>>,
                T::Error: ::std::fmt::Display,
            {
                self.itineraries = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for itineraries: {}", e));
                self
            }
            pub fn next_page_cursor<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.next_page_cursor = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for next_page_cursor: {}",
                        e
                    )
                });
                self
            }
            pub fn previous_page_cursor<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.previous_page_cursor = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for previous_page_cursor: {}",
                        e
                    )
                });
                self
            }
            pub fn request_parameters<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::collections::HashMap<::std::string::String, ::std::string::String>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.request_parameters = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for request_parameters: {}",
                        e
                    )
                });
                self
            }
            pub fn to<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.to = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for to: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<PlanResponse> for super::PlanResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: PlanResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    debug_output: value.debug_output?,
                    direct: value.direct?,
                    from: value.from?,
                    itineraries: value.itineraries?,
                    next_page_cursor: value.next_page_cursor?,
                    previous_page_cursor: value.previous_page_cursor?,
                    request_parameters: value.request_parameters?,
                    to: value.to?,
                })
            }
        }

        impl ::std::convert::From<super::PlanResponse> for PlanResponse {
            fn from(value: super::PlanResponse) -> Self {
                Self {
                    debug_output: Ok(value.debug_output),
                    direct: Ok(value.direct),
                    from: Ok(value.from),
                    itineraries: Ok(value.itineraries),
                    next_page_cursor: Ok(value.next_page_cursor),
                    previous_page_cursor: Ok(value.previous_page_cursor),
                    request_parameters: Ok(value.request_parameters),
                    to: Ok(value.to),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Reachable {
            all: ::std::result::Result<
                ::std::vec::Vec<super::ReachablePlace>,
                ::std::string::String,
            >,
            one: ::std::result::Result<::std::option::Option<super::Place>, ::std::string::String>,
        }

        impl ::std::default::Default for Reachable {
            fn default() -> Self {
                Self {
                    all: Ok(Default::default()),
                    one: Ok(Default::default()),
                }
            }
        }

        impl Reachable {
            pub fn all<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::ReachablePlace>>,
                T::Error: ::std::fmt::Display,
            {
                self.all = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for all: {}", e));
                self
            }
            pub fn one<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::Place>>,
                T::Error: ::std::fmt::Display,
            {
                self.one = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for one: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<Reachable> for super::Reachable {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Reachable,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    all: value.all?,
                    one: value.one?,
                })
            }
        }

        impl ::std::convert::From<super::Reachable> for Reachable {
            fn from(value: super::Reachable) -> Self {
                Self {
                    all: Ok(value.all),
                    one: Ok(value.one),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct ReachablePlace {
            duration: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
            k: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
            place:
                ::std::result::Result<::std::option::Option<super::Place>, ::std::string::String>,
        }

        impl ::std::default::Default for ReachablePlace {
            fn default() -> Self {
                Self {
                    duration: Ok(Default::default()),
                    k: Ok(Default::default()),
                    place: Ok(Default::default()),
                }
            }
        }

        impl ReachablePlace {
            pub fn duration<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.duration = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for duration: {}", e));
                self
            }
            pub fn k<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.k = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for k: {}", e));
                self
            }
            pub fn place<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::Place>>,
                T::Error: ::std::fmt::Display,
            {
                self.place = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for place: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<ReachablePlace> for super::ReachablePlace {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ReachablePlace,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    duration: value.duration?,
                    k: value.k?,
                    place: value.place?,
                })
            }
        }

        impl ::std::convert::From<super::ReachablePlace> for ReachablePlace {
            fn from(value: super::ReachablePlace) -> Self {
                Self {
                    duration: Ok(value.duration),
                    k: Ok(value.k),
                    place: Ok(value.place),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Rental {
            form_factor: ::std::result::Result<
                ::std::option::Option<super::RentalFormFactor>,
                ::std::string::String,
            >,
            from_station_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            propulsion_type: ::std::result::Result<
                ::std::option::Option<super::RentalPropulsionType>,
                ::std::string::String,
            >,
            rental_uri_android: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            rental_uri_ios: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            rental_uri_web: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            return_constraint: ::std::result::Result<
                ::std::option::Option<super::RentalReturnConstraint>,
                ::std::string::String,
            >,
            station_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            system_id: ::std::result::Result<::std::string::String, ::std::string::String>,
            system_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            to_station_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            url: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for Rental {
            fn default() -> Self {
                Self {
                    form_factor: Ok(Default::default()),
                    from_station_name: Ok(Default::default()),
                    propulsion_type: Ok(Default::default()),
                    rental_uri_android: Ok(Default::default()),
                    rental_uri_ios: Ok(Default::default()),
                    rental_uri_web: Ok(Default::default()),
                    return_constraint: Ok(Default::default()),
                    station_name: Ok(Default::default()),
                    system_id: Err("no value supplied for system_id".to_string()),
                    system_name: Ok(Default::default()),
                    to_station_name: Ok(Default::default()),
                    url: Ok(Default::default()),
                }
            }
        }

        impl Rental {
            pub fn form_factor<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::RentalFormFactor>>,
                T::Error: ::std::fmt::Display,
            {
                self.form_factor = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for form_factor: {}", e));
                self
            }
            pub fn from_station_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.from_station_name = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for from_station_name: {}",
                        e
                    )
                });
                self
            }
            pub fn propulsion_type<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::RentalPropulsionType>>,
                T::Error: ::std::fmt::Display,
            {
                self.propulsion_type = value.try_into().map_err(|e| {
                    format!("error converting supplied value for propulsion_type: {}", e)
                });
                self
            }
            pub fn rental_uri_android<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.rental_uri_android = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for rental_uri_android: {}",
                        e
                    )
                });
                self
            }
            pub fn rental_uri_ios<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.rental_uri_ios = value.try_into().map_err(|e| {
                    format!("error converting supplied value for rental_uri_ios: {}", e)
                });
                self
            }
            pub fn rental_uri_web<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.rental_uri_web = value.try_into().map_err(|e| {
                    format!("error converting supplied value for rental_uri_web: {}", e)
                });
                self
            }
            pub fn return_constraint<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<super::RentalReturnConstraint>>,
                T::Error: ::std::fmt::Display,
            {
                self.return_constraint = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for return_constraint: {}",
                        e
                    )
                });
                self
            }
            pub fn station_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.station_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for station_name: {}", e)
                });
                self
            }
            pub fn system_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.system_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for system_id: {}", e));
                self
            }
            pub fn system_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.system_name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for system_name: {}", e));
                self
            }
            pub fn to_station_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.to_station_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for to_station_name: {}", e)
                });
                self
            }
            pub fn url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.url = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for url: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<Rental> for super::Rental {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Rental,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    form_factor: value.form_factor?,
                    from_station_name: value.from_station_name?,
                    propulsion_type: value.propulsion_type?,
                    rental_uri_android: value.rental_uri_android?,
                    rental_uri_ios: value.rental_uri_ios?,
                    rental_uri_web: value.rental_uri_web?,
                    return_constraint: value.return_constraint?,
                    station_name: value.station_name?,
                    system_id: value.system_id?,
                    system_name: value.system_name?,
                    to_station_name: value.to_station_name?,
                    url: value.url?,
                })
            }
        }

        impl ::std::convert::From<super::Rental> for Rental {
            fn from(value: super::Rental) -> Self {
                Self {
                    form_factor: Ok(value.form_factor),
                    from_station_name: Ok(value.from_station_name),
                    propulsion_type: Ok(value.propulsion_type),
                    rental_uri_android: Ok(value.rental_uri_android),
                    rental_uri_ios: Ok(value.rental_uri_ios),
                    rental_uri_web: Ok(value.rental_uri_web),
                    return_constraint: Ok(value.return_constraint),
                    station_name: Ok(value.station_name),
                    system_id: Ok(value.system_id),
                    system_name: Ok(value.system_name),
                    to_station_name: Ok(value.to_station_name),
                    url: Ok(value.url),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct RiderCategory {
            eligibility_url: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            is_default_fare_category: ::std::result::Result<bool, ::std::string::String>,
            rider_category_name:
                ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for RiderCategory {
            fn default() -> Self {
                Self {
                    eligibility_url: Ok(Default::default()),
                    is_default_fare_category: Err(
                        "no value supplied for is_default_fare_category".to_string()
                    ),
                    rider_category_name: Err(
                        "no value supplied for rider_category_name".to_string()
                    ),
                }
            }
        }

        impl RiderCategory {
            pub fn eligibility_url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.eligibility_url = value.try_into().map_err(|e| {
                    format!("error converting supplied value for eligibility_url: {}", e)
                });
                self
            }
            pub fn is_default_fare_category<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.is_default_fare_category = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for is_default_fare_category: {}",
                        e
                    )
                });
                self
            }
            pub fn rider_category_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.rider_category_name = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for rider_category_name: {}",
                        e
                    )
                });
                self
            }
        }

        impl ::std::convert::TryFrom<RiderCategory> for super::RiderCategory {
            type Error = super::error::ConversionError;
            fn try_from(
                value: RiderCategory,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    eligibility_url: value.eligibility_url?,
                    is_default_fare_category: value.is_default_fare_category?,
                    rider_category_name: value.rider_category_name?,
                })
            }
        }

        impl ::std::convert::From<super::RiderCategory> for RiderCategory {
            fn from(value: super::RiderCategory) -> Self {
                Self {
                    eligibility_url: Ok(value.eligibility_url),
                    is_default_fare_category: Ok(value.is_default_fare_category),
                    rider_category_name: Ok(value.rider_category_name),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct StepInstruction {
            access_restriction: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            area: ::std::result::Result<bool, ::std::string::String>,
            distance: ::std::result::Result<f64, ::std::string::String>,
            elevation_down:
                ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
            elevation_up: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
            exit: ::std::result::Result<::std::string::String, ::std::string::String>,
            from_level: ::std::result::Result<f64, ::std::string::String>,
            osm_way: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
            polyline: ::std::result::Result<super::EncodedPolyline, ::std::string::String>,
            relative_direction: ::std::result::Result<super::Direction, ::std::string::String>,
            stay_on: ::std::result::Result<bool, ::std::string::String>,
            street_name: ::std::result::Result<::std::string::String, ::std::string::String>,
            to_level: ::std::result::Result<f64, ::std::string::String>,
            toll: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        }

        impl ::std::default::Default for StepInstruction {
            fn default() -> Self {
                Self {
                    access_restriction: Ok(Default::default()),
                    area: Err("no value supplied for area".to_string()),
                    distance: Err("no value supplied for distance".to_string()),
                    elevation_down: Ok(Default::default()),
                    elevation_up: Ok(Default::default()),
                    exit: Err("no value supplied for exit".to_string()),
                    from_level: Err("no value supplied for from_level".to_string()),
                    osm_way: Ok(Default::default()),
                    polyline: Err("no value supplied for polyline".to_string()),
                    relative_direction: Err("no value supplied for relative_direction".to_string()),
                    stay_on: Err("no value supplied for stay_on".to_string()),
                    street_name: Err("no value supplied for street_name".to_string()),
                    to_level: Err("no value supplied for to_level".to_string()),
                    toll: Ok(Default::default()),
                }
            }
        }

        impl StepInstruction {
            pub fn access_restriction<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.access_restriction = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for access_restriction: {}",
                        e
                    )
                });
                self
            }
            pub fn area<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.area = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for area: {}", e));
                self
            }
            pub fn distance<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.distance = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for distance: {}", e));
                self
            }
            pub fn elevation_down<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.elevation_down = value.try_into().map_err(|e| {
                    format!("error converting supplied value for elevation_down: {}", e)
                });
                self
            }
            pub fn elevation_up<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.elevation_up = value.try_into().map_err(|e| {
                    format!("error converting supplied value for elevation_up: {}", e)
                });
                self
            }
            pub fn exit<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.exit = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for exit: {}", e));
                self
            }
            pub fn from_level<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.from_level = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for from_level: {}", e));
                self
            }
            pub fn osm_way<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.osm_way = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for osm_way: {}", e));
                self
            }
            pub fn polyline<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::EncodedPolyline>,
                T::Error: ::std::fmt::Display,
            {
                self.polyline = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for polyline: {}", e));
                self
            }
            pub fn relative_direction<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Direction>,
                T::Error: ::std::fmt::Display,
            {
                self.relative_direction = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for relative_direction: {}",
                        e
                    )
                });
                self
            }
            pub fn stay_on<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.stay_on = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for stay_on: {}", e));
                self
            }
            pub fn street_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.street_name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for street_name: {}", e));
                self
            }
            pub fn to_level<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.to_level = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for to_level: {}", e));
                self
            }
            pub fn toll<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<bool>>,
                T::Error: ::std::fmt::Display,
            {
                self.toll = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for toll: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<StepInstruction> for super::StepInstruction {
            type Error = super::error::ConversionError;
            fn try_from(
                value: StepInstruction,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    access_restriction: value.access_restriction?,
                    area: value.area?,
                    distance: value.distance?,
                    elevation_down: value.elevation_down?,
                    elevation_up: value.elevation_up?,
                    exit: value.exit?,
                    from_level: value.from_level?,
                    osm_way: value.osm_way?,
                    polyline: value.polyline?,
                    relative_direction: value.relative_direction?,
                    stay_on: value.stay_on?,
                    street_name: value.street_name?,
                    to_level: value.to_level?,
                    toll: value.toll?,
                })
            }
        }

        impl ::std::convert::From<super::StepInstruction> for StepInstruction {
            fn from(value: super::StepInstruction) -> Self {
                Self {
                    access_restriction: Ok(value.access_restriction),
                    area: Ok(value.area),
                    distance: Ok(value.distance),
                    elevation_down: Ok(value.elevation_down),
                    elevation_up: Ok(value.elevation_up),
                    exit: Ok(value.exit),
                    from_level: Ok(value.from_level),
                    osm_way: Ok(value.osm_way),
                    polyline: Ok(value.polyline),
                    relative_direction: Ok(value.relative_direction),
                    stay_on: Ok(value.stay_on),
                    street_name: Ok(value.street_name),
                    to_level: Ok(value.to_level),
                    toll: Ok(value.toll),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct StopTime {
            agency_id: ::std::result::Result<::std::string::String, ::std::string::String>,
            agency_name: ::std::result::Result<::std::string::String, ::std::string::String>,
            agency_url: ::std::result::Result<::std::string::String, ::std::string::String>,
            cancelled: ::std::result::Result<bool, ::std::string::String>,
            display_name: ::std::result::Result<::std::string::String, ::std::string::String>,
            headsign: ::std::result::Result<::std::string::String, ::std::string::String>,
            mode: ::std::result::Result<super::Mode, ::std::string::String>,
            next_stops: ::std::result::Result<::std::vec::Vec<super::Place>, ::std::string::String>,
            pickup_dropoff_type:
                ::std::result::Result<super::PickupDropoffType, ::std::string::String>,
            place: ::std::result::Result<super::Place, ::std::string::String>,
            previous_stops:
                ::std::result::Result<::std::vec::Vec<super::Place>, ::std::string::String>,
            real_time: ::std::result::Result<bool, ::std::string::String>,
            route_color: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            route_long_name: ::std::result::Result<::std::string::String, ::std::string::String>,
            route_short_name: ::std::result::Result<::std::string::String, ::std::string::String>,
            route_text_color: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            route_type: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
            source: ::std::result::Result<::std::string::String, ::std::string::String>,
            trip_cancelled: ::std::result::Result<bool, ::std::string::String>,
            trip_id: ::std::result::Result<::std::string::String, ::std::string::String>,
            trip_short_name: ::std::result::Result<::std::string::String, ::std::string::String>,
            trip_to: ::std::result::Result<super::Place, ::std::string::String>,
        }

        impl ::std::default::Default for StopTime {
            fn default() -> Self {
                Self {
                    agency_id: Err("no value supplied for agency_id".to_string()),
                    agency_name: Err("no value supplied for agency_name".to_string()),
                    agency_url: Err("no value supplied for agency_url".to_string()),
                    cancelled: Err("no value supplied for cancelled".to_string()),
                    display_name: Err("no value supplied for display_name".to_string()),
                    headsign: Err("no value supplied for headsign".to_string()),
                    mode: Err("no value supplied for mode".to_string()),
                    next_stops: Ok(Default::default()),
                    pickup_dropoff_type: Err(
                        "no value supplied for pickup_dropoff_type".to_string()
                    ),
                    place: Err("no value supplied for place".to_string()),
                    previous_stops: Ok(Default::default()),
                    real_time: Err("no value supplied for real_time".to_string()),
                    route_color: Ok(Default::default()),
                    route_long_name: Err("no value supplied for route_long_name".to_string()),
                    route_short_name: Err("no value supplied for route_short_name".to_string()),
                    route_text_color: Ok(Default::default()),
                    route_type: Ok(Default::default()),
                    source: Err("no value supplied for source".to_string()),
                    trip_cancelled: Err("no value supplied for trip_cancelled".to_string()),
                    trip_id: Err("no value supplied for trip_id".to_string()),
                    trip_short_name: Err("no value supplied for trip_short_name".to_string()),
                    trip_to: Err("no value supplied for trip_to".to_string()),
                }
            }
        }

        impl StopTime {
            pub fn agency_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.agency_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for agency_id: {}", e));
                self
            }
            pub fn agency_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.agency_name = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for agency_name: {}", e));
                self
            }
            pub fn agency_url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.agency_url = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for agency_url: {}", e));
                self
            }
            pub fn cancelled<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.cancelled = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for cancelled: {}", e));
                self
            }
            pub fn display_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.display_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for display_name: {}", e)
                });
                self
            }
            pub fn headsign<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.headsign = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for headsign: {}", e));
                self
            }
            pub fn mode<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Mode>,
                T::Error: ::std::fmt::Display,
            {
                self.mode = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for mode: {}", e));
                self
            }
            pub fn next_stops<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Place>>,
                T::Error: ::std::fmt::Display,
            {
                self.next_stops = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for next_stops: {}", e));
                self
            }
            pub fn pickup_dropoff_type<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::PickupDropoffType>,
                T::Error: ::std::fmt::Display,
            {
                self.pickup_dropoff_type = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for pickup_dropoff_type: {}",
                        e
                    )
                });
                self
            }
            pub fn place<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.place = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for place: {}", e));
                self
            }
            pub fn previous_stops<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Place>>,
                T::Error: ::std::fmt::Display,
            {
                self.previous_stops = value.try_into().map_err(|e| {
                    format!("error converting supplied value for previous_stops: {}", e)
                });
                self
            }
            pub fn real_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.real_time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for real_time: {}", e));
                self
            }
            pub fn route_color<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_color = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for route_color: {}", e));
                self
            }
            pub fn route_long_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.route_long_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for route_long_name: {}", e)
                });
                self
            }
            pub fn route_short_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.route_short_name = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for route_short_name: {}",
                        e
                    )
                });
                self
            }
            pub fn route_text_color<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_text_color = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for route_text_color: {}",
                        e
                    )
                });
                self
            }
            pub fn route_type<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<i64>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_type = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for route_type: {}", e));
                self
            }
            pub fn source<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.source = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for source: {}", e));
                self
            }
            pub fn trip_cancelled<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.trip_cancelled = value.try_into().map_err(|e| {
                    format!("error converting supplied value for trip_cancelled: {}", e)
                });
                self
            }
            pub fn trip_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.trip_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for trip_id: {}", e));
                self
            }
            pub fn trip_short_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.trip_short_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for trip_short_name: {}", e)
                });
                self
            }
            pub fn trip_to<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.trip_to = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for trip_to: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<StopTime> for super::StopTime {
            type Error = super::error::ConversionError;
            fn try_from(
                value: StopTime,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    agency_id: value.agency_id?,
                    agency_name: value.agency_name?,
                    agency_url: value.agency_url?,
                    cancelled: value.cancelled?,
                    display_name: value.display_name?,
                    headsign: value.headsign?,
                    mode: value.mode?,
                    next_stops: value.next_stops?,
                    pickup_dropoff_type: value.pickup_dropoff_type?,
                    place: value.place?,
                    previous_stops: value.previous_stops?,
                    real_time: value.real_time?,
                    route_color: value.route_color?,
                    route_long_name: value.route_long_name?,
                    route_short_name: value.route_short_name?,
                    route_text_color: value.route_text_color?,
                    route_type: value.route_type?,
                    source: value.source?,
                    trip_cancelled: value.trip_cancelled?,
                    trip_id: value.trip_id?,
                    trip_short_name: value.trip_short_name?,
                    trip_to: value.trip_to?,
                })
            }
        }

        impl ::std::convert::From<super::StopTime> for StopTime {
            fn from(value: super::StopTime) -> Self {
                Self {
                    agency_id: Ok(value.agency_id),
                    agency_name: Ok(value.agency_name),
                    agency_url: Ok(value.agency_url),
                    cancelled: Ok(value.cancelled),
                    display_name: Ok(value.display_name),
                    headsign: Ok(value.headsign),
                    mode: Ok(value.mode),
                    next_stops: Ok(value.next_stops),
                    pickup_dropoff_type: Ok(value.pickup_dropoff_type),
                    place: Ok(value.place),
                    previous_stops: Ok(value.previous_stops),
                    real_time: Ok(value.real_time),
                    route_color: Ok(value.route_color),
                    route_long_name: Ok(value.route_long_name),
                    route_short_name: Ok(value.route_short_name),
                    route_text_color: Ok(value.route_text_color),
                    route_type: Ok(value.route_type),
                    source: Ok(value.source),
                    trip_cancelled: Ok(value.trip_cancelled),
                    trip_id: Ok(value.trip_id),
                    trip_short_name: Ok(value.trip_short_name),
                    trip_to: Ok(value.trip_to),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct StoptimesResponse {
            next_page_cursor: ::std::result::Result<::std::string::String, ::std::string::String>,
            place: ::std::result::Result<super::Place, ::std::string::String>,
            previous_page_cursor:
                ::std::result::Result<::std::string::String, ::std::string::String>,
            stop_times:
                ::std::result::Result<::std::vec::Vec<super::StopTime>, ::std::string::String>,
        }

        impl ::std::default::Default for StoptimesResponse {
            fn default() -> Self {
                Self {
                    next_page_cursor: Err("no value supplied for next_page_cursor".to_string()),
                    place: Err("no value supplied for place".to_string()),
                    previous_page_cursor: Err(
                        "no value supplied for previous_page_cursor".to_string()
                    ),
                    stop_times: Err("no value supplied for stop_times".to_string()),
                }
            }
        }

        impl StoptimesResponse {
            pub fn next_page_cursor<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.next_page_cursor = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for next_page_cursor: {}",
                        e
                    )
                });
                self
            }
            pub fn place<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.place = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for place: {}", e));
                self
            }
            pub fn previous_page_cursor<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.previous_page_cursor = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for previous_page_cursor: {}",
                        e
                    )
                });
                self
            }
            pub fn stop_times<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::StopTime>>,
                T::Error: ::std::fmt::Display,
            {
                self.stop_times = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for stop_times: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<StoptimesResponse> for super::StoptimesResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: StoptimesResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    next_page_cursor: value.next_page_cursor?,
                    place: value.place?,
                    previous_page_cursor: value.previous_page_cursor?,
                    stop_times: value.stop_times?,
                })
            }
        }

        impl ::std::convert::From<super::StoptimesResponse> for StoptimesResponse {
            fn from(value: super::StoptimesResponse) -> Self {
                Self {
                    next_page_cursor: Ok(value.next_page_cursor),
                    place: Ok(value.place),
                    previous_page_cursor: Ok(value.previous_page_cursor),
                    stop_times: Ok(value.stop_times),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct TimeRange {
            end: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
            start: ::std::result::Result<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for TimeRange {
            fn default() -> Self {
                Self {
                    end: Ok(Default::default()),
                    start: Ok(Default::default()),
                }
            }
        }

        impl TimeRange {
            pub fn end<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.end = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for end: {}", e));
                self
            }
            pub fn start<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<
                        ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
                    >,
                T::Error: ::std::fmt::Display,
            {
                self.start = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for start: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<TimeRange> for super::TimeRange {
            type Error = super::error::ConversionError;
            fn try_from(
                value: TimeRange,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    end: value.end?,
                    start: value.start?,
                })
            }
        }

        impl ::std::convert::From<super::TimeRange> for TimeRange {
            fn from(value: super::TimeRange) -> Self {
                Self {
                    end: Ok(value.end),
                    start: Ok(value.start),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct Transfer {
            car: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
            default: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
            foot: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
            foot_routed: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
            to: ::std::result::Result<super::Place, ::std::string::String>,
            wheelchair: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
            wheelchair_routed:
                ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
            wheelchair_uses_elevator:
                ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        }

        impl ::std::default::Default for Transfer {
            fn default() -> Self {
                Self {
                    car: Ok(Default::default()),
                    default: Ok(Default::default()),
                    foot: Ok(Default::default()),
                    foot_routed: Ok(Default::default()),
                    to: Err("no value supplied for to".to_string()),
                    wheelchair: Ok(Default::default()),
                    wheelchair_routed: Ok(Default::default()),
                    wheelchair_uses_elevator: Ok(Default::default()),
                }
            }
        }

        impl Transfer {
            pub fn car<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<f64>>,
                T::Error: ::std::fmt::Display,
            {
                self.car = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for car: {}", e));
                self
            }
            pub fn default<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<f64>>,
                T::Error: ::std::fmt::Display,
            {
                self.default = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for default: {}", e));
                self
            }
            pub fn foot<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<f64>>,
                T::Error: ::std::fmt::Display,
            {
                self.foot = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for foot: {}", e));
                self
            }
            pub fn foot_routed<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<f64>>,
                T::Error: ::std::fmt::Display,
            {
                self.foot_routed = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for foot_routed: {}", e));
                self
            }
            pub fn to<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.to = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for to: {}", e));
                self
            }
            pub fn wheelchair<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<f64>>,
                T::Error: ::std::fmt::Display,
            {
                self.wheelchair = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for wheelchair: {}", e));
                self
            }
            pub fn wheelchair_routed<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<f64>>,
                T::Error: ::std::fmt::Display,
            {
                self.wheelchair_routed = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for wheelchair_routed: {}",
                        e
                    )
                });
                self
            }
            pub fn wheelchair_uses_elevator<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<bool>>,
                T::Error: ::std::fmt::Display,
            {
                self.wheelchair_uses_elevator = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for wheelchair_uses_elevator: {}",
                        e
                    )
                });
                self
            }
        }

        impl ::std::convert::TryFrom<Transfer> for super::Transfer {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Transfer,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    car: value.car?,
                    default: value.default?,
                    foot: value.foot?,
                    foot_routed: value.foot_routed?,
                    to: value.to?,
                    wheelchair: value.wheelchair?,
                    wheelchair_routed: value.wheelchair_routed?,
                    wheelchair_uses_elevator: value.wheelchair_uses_elevator?,
                })
            }
        }

        impl ::std::convert::From<super::Transfer> for Transfer {
            fn from(value: super::Transfer) -> Self {
                Self {
                    car: Ok(value.car),
                    default: Ok(value.default),
                    foot: Ok(value.foot),
                    foot_routed: Ok(value.foot_routed),
                    to: Ok(value.to),
                    wheelchair: Ok(value.wheelchair),
                    wheelchair_routed: Ok(value.wheelchair_routed),
                    wheelchair_uses_elevator: Ok(value.wheelchair_uses_elevator),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct TransfersResponse {
            has_car_transfers: ::std::result::Result<bool, ::std::string::String>,
            has_foot_transfers: ::std::result::Result<bool, ::std::string::String>,
            has_wheelchair_transfers: ::std::result::Result<bool, ::std::string::String>,
            place: ::std::result::Result<super::Place, ::std::string::String>,
            transfers:
                ::std::result::Result<::std::vec::Vec<super::Transfer>, ::std::string::String>,
        }

        impl ::std::default::Default for TransfersResponse {
            fn default() -> Self {
                Self {
                    has_car_transfers: Err("no value supplied for has_car_transfers".to_string()),
                    has_foot_transfers: Err("no value supplied for has_foot_transfers".to_string()),
                    has_wheelchair_transfers: Err(
                        "no value supplied for has_wheelchair_transfers".to_string()
                    ),
                    place: Err("no value supplied for place".to_string()),
                    transfers: Err("no value supplied for transfers".to_string()),
                }
            }
        }

        impl TransfersResponse {
            pub fn has_car_transfers<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.has_car_transfers = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for has_car_transfers: {}",
                        e
                    )
                });
                self
            }
            pub fn has_foot_transfers<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.has_foot_transfers = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for has_foot_transfers: {}",
                        e
                    )
                });
                self
            }
            pub fn has_wheelchair_transfers<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.has_wheelchair_transfers = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for has_wheelchair_transfers: {}",
                        e
                    )
                });
                self
            }
            pub fn place<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.place = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for place: {}", e));
                self
            }
            pub fn transfers<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::Transfer>>,
                T::Error: ::std::fmt::Display,
            {
                self.transfers = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for transfers: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<TransfersResponse> for super::TransfersResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: TransfersResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    has_car_transfers: value.has_car_transfers?,
                    has_foot_transfers: value.has_foot_transfers?,
                    has_wheelchair_transfers: value.has_wheelchair_transfers?,
                    place: value.place?,
                    transfers: value.transfers?,
                })
            }
        }

        impl ::std::convert::From<super::TransfersResponse> for TransfersResponse {
            fn from(value: super::TransfersResponse) -> Self {
                Self {
                    has_car_transfers: Ok(value.has_car_transfers),
                    has_foot_transfers: Ok(value.has_foot_transfers),
                    has_wheelchair_transfers: Ok(value.has_wheelchair_transfers),
                    place: Ok(value.place),
                    transfers: Ok(value.transfers),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct TripInfo {
            display_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            route_short_name: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            trip_id: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for TripInfo {
            fn default() -> Self {
                Self {
                    display_name: Ok(Default::default()),
                    route_short_name: Ok(Default::default()),
                    trip_id: Err("no value supplied for trip_id".to_string()),
                }
            }
        }

        impl TripInfo {
            pub fn display_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.display_name = value.try_into().map_err(|e| {
                    format!("error converting supplied value for display_name: {}", e)
                });
                self
            }
            pub fn route_short_name<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_short_name = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for route_short_name: {}",
                        e
                    )
                });
                self
            }
            pub fn trip_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.trip_id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for trip_id: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<TripInfo> for super::TripInfo {
            type Error = super::error::ConversionError;
            fn try_from(
                value: TripInfo,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    display_name: value.display_name?,
                    route_short_name: value.route_short_name?,
                    trip_id: value.trip_id?,
                })
            }
        }

        impl ::std::convert::From<super::TripInfo> for TripInfo {
            fn from(value: super::TripInfo) -> Self {
                Self {
                    display_name: Ok(value.display_name),
                    route_short_name: Ok(value.route_short_name),
                    trip_id: Ok(value.trip_id),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct TripSegment {
            arrival: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            departure: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            distance: ::std::result::Result<f64, ::std::string::String>,
            from: ::std::result::Result<super::Place, ::std::string::String>,
            mode: ::std::result::Result<super::Mode, ::std::string::String>,
            polyline: ::std::result::Result<::std::string::String, ::std::string::String>,
            real_time: ::std::result::Result<bool, ::std::string::String>,
            route_color: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            scheduled_arrival: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            scheduled_departure: ::std::result::Result<
                ::chrono::DateTime<::chrono::offset::Utc>,
                ::std::string::String,
            >,
            to: ::std::result::Result<super::Place, ::std::string::String>,
            trips: ::std::result::Result<::std::vec::Vec<super::TripInfo>, ::std::string::String>,
        }

        impl ::std::default::Default for TripSegment {
            fn default() -> Self {
                Self {
                    arrival: Err("no value supplied for arrival".to_string()),
                    departure: Err("no value supplied for departure".to_string()),
                    distance: Err("no value supplied for distance".to_string()),
                    from: Err("no value supplied for from".to_string()),
                    mode: Err("no value supplied for mode".to_string()),
                    polyline: Err("no value supplied for polyline".to_string()),
                    real_time: Err("no value supplied for real_time".to_string()),
                    route_color: Ok(Default::default()),
                    scheduled_arrival: Err("no value supplied for scheduled_arrival".to_string()),
                    scheduled_departure: Err(
                        "no value supplied for scheduled_departure".to_string()
                    ),
                    to: Err("no value supplied for to".to_string()),
                    trips: Err("no value supplied for trips".to_string()),
                }
            }
        }

        impl TripSegment {
            pub fn arrival<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.arrival = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for arrival: {}", e));
                self
            }
            pub fn departure<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.departure = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for departure: {}", e));
                self
            }
            pub fn distance<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<f64>,
                T::Error: ::std::fmt::Display,
            {
                self.distance = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for distance: {}", e));
                self
            }
            pub fn from<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.from = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for from: {}", e));
                self
            }
            pub fn mode<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Mode>,
                T::Error: ::std::fmt::Display,
            {
                self.mode = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for mode: {}", e));
                self
            }
            pub fn polyline<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.polyline = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for polyline: {}", e));
                self
            }
            pub fn real_time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<bool>,
                T::Error: ::std::fmt::Display,
            {
                self.real_time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for real_time: {}", e));
                self
            }
            pub fn route_color<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.route_color = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for route_color: {}", e));
                self
            }
            pub fn scheduled_arrival<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.scheduled_arrival = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for scheduled_arrival: {}",
                        e
                    )
                });
                self
            }
            pub fn scheduled_departure<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
                T::Error: ::std::fmt::Display,
            {
                self.scheduled_departure = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for scheduled_departure: {}",
                        e
                    )
                });
                self
            }
            pub fn to<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Place>,
                T::Error: ::std::fmt::Display,
            {
                self.to = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for to: {}", e));
                self
            }
            pub fn trips<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::TripInfo>>,
                T::Error: ::std::fmt::Display,
            {
                self.trips = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for trips: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<TripSegment> for super::TripSegment {
            type Error = super::error::ConversionError;
            fn try_from(
                value: TripSegment,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    arrival: value.arrival?,
                    departure: value.departure?,
                    distance: value.distance?,
                    from: value.from?,
                    mode: value.mode?,
                    polyline: value.polyline?,
                    real_time: value.real_time?,
                    route_color: value.route_color?,
                    scheduled_arrival: value.scheduled_arrival?,
                    scheduled_departure: value.scheduled_departure?,
                    to: value.to?,
                    trips: value.trips?,
                })
            }
        }

        impl ::std::convert::From<super::TripSegment> for TripSegment {
            fn from(value: super::TripSegment) -> Self {
                Self {
                    arrival: Ok(value.arrival),
                    departure: Ok(value.departure),
                    distance: Ok(value.distance),
                    from: Ok(value.from),
                    mode: Ok(value.mode),
                    polyline: Ok(value.polyline),
                    real_time: Ok(value.real_time),
                    route_color: Ok(value.route_color),
                    scheduled_arrival: Ok(value.scheduled_arrival),
                    scheduled_departure: Ok(value.scheduled_departure),
                    to: Ok(value.to),
                    trips: Ok(value.trips),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
///Client for MOTIS API
///
///This is the MOTIS routing API.
///
///Version: v4
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}

impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = std::time::Duration::from_secs(15);
            reqwest::ClientBuilder::new()
                .connect_timeout(dur)
                .timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }

    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }
}

impl ClientInfo<()> for Client {
    fn api_version() -> &'static str {
        "v4"
    }

    fn baseurl(&self) -> &str {
        self.baseurl.as_str()
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn inner(&self) -> &() {
        &()
    }
}

impl ClientHooks<()> for &Client {}
impl Client {
    ///Computes optimal connections from one place to another
    ///
    ///Sends a `GET` request to `/api/v4/plan`
    ///
    ///Arguments:
    /// - `additional_transfer_time`: Optional. Default is 0 minutes.
    ///
    ///Additional transfer time reserved for each transfer in minutes.
    ///
    /// - `algorithm`: algorithm to use
    /// - `arrive_by`: Optional. Default is `false`.
    ///
    ///  - `arriveBy=true`: the parameters `date` and `time` refer to the
    ///    arrival time
    ///  - `arriveBy=false`: the parameters `date` and `time` refer to the
    ///    departure time
    ///
    /// - `detailed_transfers`: - true: Compute transfer polylines and step
    ///   instructions.
    /// - false: Only return basic information (start time, end time, duration)
    ///   for transfers.
    ///
    /// - `direct_modes`: Optional. Default is `WALK` which will compute walking
    ///   routes as direct connections.
    ///
    ///Modes used for direction connections from start to destination without
    /// using transit. Results will be returned on the `direct` key.
    ///
    ///Note: Direct connections will only be returned on the first call. For
    /// paging calls, they can be omitted.
    ///
    ///Note: Transit connections that are slower than the fastest direct
    /// connection will not show up. This is being used as a cut-off during
    /// transit routing to speed up the search. To prevent this, it's
    /// possible to send two separate requests (one with only `transitModes` and
    /// one with only `directModes`).
    ///
    ///Note: the output `direct` array will stay empty if the input param
    /// `maxDirectTime` makes any direct trip impossible.
    ///
    ///Only non-transit modes such as `WALK`, `BIKE`, `CAR`, `BIKE_SHARING`,
    /// etc. can be used.
    ///
    /// - `direct_rental_form_factors`: Experimental. Expect unannounced
    ///   breaking changes (without version bumps).
    ///
    ///Optional. Only applies to direct connections.
    ///
    ///A list of vehicle type form factors that are allowed to be used for
    /// direct connections. If empty (the default), all form factors are
    /// allowed. Example: `BICYCLE,SCOOTER_STANDING`.
    ///
    /// - `direct_rental_propulsion_types`: Experimental. Expect unannounced
    ///   breaking changes (without version bumps).
    ///
    ///Optional. Only applies to direct connections.
    ///
    ///A list of vehicle type form factors that are allowed to be used for
    /// direct connections. If empty (the default), all propulsion types are
    /// allowed. Example: `HUMAN,ELECTRIC,ELECTRIC_ASSIST`.
    ///
    /// - `direct_rental_providers`: Experimental. Expect unannounced breaking
    ///   changes (without version bumps).
    ///
    ///Optional. Only applies to direct connections.
    ///
    ///A list of rental providers that are allowed to be used for direct
    /// connections. If empty (the default), all providers are allowed.
    ///
    /// - `elevation_costs`: Optional. Default is `NONE`.
    ///
    ///Set an elevation cost profile, to penalize routes with incline.
    /// - `NONE`: No additional costs for elevations. This is the default
    ///   behavior
    /// - `LOW`: Add a low cost for increase in elevation and incline along the
    ///   way. This will prefer routes with less ascent, if small detours are
    ///   required.
    /// - `HIGH`: Add a high cost for increase in elevation and incline along
    ///   the way. This will prefer routes with less ascent, if larger detours
    ///   are required.
    ///
    ///As using an elevation costs profile will increase the travel duration,
    ///routing through steep terrain may exceed the maximal allowed duration,
    ///causing a location to appear unreachable.
    ///Increasing the maximum travel time for these segments may resolve this
    /// issue.
    ///
    ///The profile is used for direct routing, on the first mile, and last
    /// mile.
    ///
    ///Elevation cost profiles are currently used by following street modes:
    /// - `BIKE`
    ///
    /// - `fastest_direct_factor`: Optional. Experimental. Default is `1.0`.
    ///Factor with which the duration of the fastest direct non-public-transit
    /// connection is multiplied. Values > 1.0 allow transit connections
    /// that are slower than the fastest direct non-public-transit connection to
    /// be found.
    ///
    /// - `fastest_slow_direct_factor`: Optional.
    ///Factor with which the duration of the fastest slowDirect connection is
    /// multiplied. Values > 1.0 allow connections that are slower than the
    /// fastest direct transit connection to be found. Values < 1.0 will
    /// return all slowDirect connections.
    ///
    /// - `from_place`: \`latitude,longitude[,level]\` tuple with
    /// - latitude and longitude in degrees
    /// - (optional) level: the OSM level (default: 0)
    ///
    ///OR
    ///
    ///stop id
    ///
    /// - `ignore_direct_rental_return_constraints`: Experimental. Expect
    ///   unannounced breaking changes (without version bumps).
    ///
    ///Optional. Default is `false`.
    ///
    ///If set to `true`, the routing will ignore rental return constraints for
    /// direct connections, allowing the rental vehicle to be parked
    /// anywhere.
    ///
    /// - `ignore_post_transit_rental_return_constraints`: Experimental. Expect
    ///   unannounced breaking changes (without version bumps).
    ///
    ///Optional. Default is `false`.
    ///
    ///If set to `true`, the routing will ignore rental return constraints for
    /// the part from the last transit stop to the `to` coordinate, allowing
    /// the rental vehicle to be parked anywhere.
    ///
    /// - `ignore_pre_transit_rental_return_constraints`: Experimental. Expect
    ///   unannounced breaking changes (without version bumps).
    ///
    ///Optional. Default is `false`.
    ///
    ///If set to `true`, the routing will ignore rental return constraints for
    /// the part from the `from` coordinate to the first transit stop,
    /// allowing the rental vehicle to be parked anywhere.
    ///
    /// - `join_interlined_legs`: Optional. Default is `true`.
    ///
    ///Controls if a journey section with stay-seated transfers is returned:
    /// - `joinInterlinedLegs=false`: as several legs (full information about
    ///   all trip numbers, headsigns, etc.).
    ///  Legs that do not require a transfer (stay-seated transfer) are marked
    /// with `interlineWithPreviousLeg=true`.
    /// - `joinInterlinedLegs=true` (default behavior): as only one joined leg
    ///   containing all stops
    ///
    /// - `language`: language tags as used in OpenStreetMap / GTFS
    ///(usually BCP-47 / ISO 639-1, or ISO 639-2 if there's no ISO 639-1)
    ///
    /// - `luggage`: Optional. Experimental. Number of luggage pieces; base
    ///   unit: airline cabin luggage (e.g. for ODM or price calculation)
    ///
    /// - `max_direct_time`: Optional. Default is 30min which is `1800`.
    ///Maximum time in seconds for direct connections.
    ///
    /// - `max_matching_distance`: Optional. Default is 25 meters.
    ///
    ///Maximum matching distance in meters to match geo coordinates to the
    /// street network.
    ///
    /// - `max_post_transit_time`: Optional. Default is 15min which is `900`.
    ///Maximum time in seconds for the last street leg.
    ///
    /// - `max_pre_transit_time`: Optional. Default is 15min which is `900`.
    ///Maximum time in seconds for the first street leg.
    ///
    /// - `max_transfers`: The maximum number of allowed transfers (i.e.
    ///   interchanges between transit legs,
    ///pre- and postTransit do not count as transfers).
    ///`maxTransfers=0` searches for direct transit connections without any
    /// transfers. If you want to search only for non-transit connections
    /// (`FOOT`, `CAR`, etc.), send an empty `transitModes` parameter
    /// instead.
    ///
    ///If not provided, the routing uses the server-side default value
    ///which is hardcoded and very high to cover all use cases.
    ///
    ///*Warning*: Use with care. Setting this too low can lead to
    ///optimal (e.g. the fastest) journeys not being found.
    ///If this value is too low to reach the destination at all,
    ///it can lead to slow routing performance.
    ///
    ///In plan endpoints before v3, the behavior is off by one,
    ///i.e. `maxTransfers=0` only returns non-transit connections.
    ///
    /// - `max_travel_time`: The maximum travel time in minutes.
    ///If not provided, the routing to uses the value
    ///hardcoded in the server which is usually quite high.
    ///
    ///*Warning*: Use with care. Setting this too low can lead to
    ///optimal (e.g. the least transfers) journeys not being found.
    ///If this value is too low to reach the destination at all,
    ///it can lead to slow routing performance.
    ///
    /// - `min_transfer_time`: Optional. Default is 0 minutes.
    ///
    ///Minimum transfer time for each transfer in minutes.
    ///
    /// - `num_itineraries`: The minimum number of itineraries to compute.
    ///This is only relevant if `timetableView=true`.
    ///The default value is 5.
    ///
    /// - `page_cursor`: Use the cursor to go to the next "page" of itineraries.
    ///Copy the cursor from the last response and keep the original request as
    /// is. This will enable you to search for itineraries in the next or
    /// previous time-window.
    ///
    /// - `passengers`: Optional. Experimental. Number of passengers (e.g. for
    ///   ODM or price calculation)
    /// - `pedestrian_profile`: Optional. Default is `FOOT`.
    ///
    ///Accessibility profile to use for pedestrian routing in transfers
    ///between transit connections, on the first mile, and last mile.
    ///
    /// - `post_transit_modes`: Optional. Default is `WALK`. Only applies if the
    ///   `to` place is a coordinate (not a transit stop). Does not apply to
    ///   direct connections (see `directModes`).
    ///
    ///A list of modes that are allowed to be used from the last transit stop
    /// to the `to` coordinate. Example: `WALK,BIKE_SHARING`.
    ///
    /// - `post_transit_rental_form_factors`: Experimental. Expect unannounced
    ///   breaking changes (without version bumps).
    ///
    ///Optional. Only applies if the `to` place is a coordinate (not a transit
    /// stop). Does not apply to direct connections (see
    /// `directRentalFormFactors`).
    ///
    ///A list of vehicle type form factors that are allowed to be used from the
    /// last transit stop to the `to` coordinate. If empty (the default),
    /// all form factors are allowed. Example: `BICYCLE,SCOOTER_STANDING`.
    ///
    /// - `post_transit_rental_propulsion_types`: Experimental. Expect
    ///   unannounced breaking changes (without version bumps).
    ///
    ///Optional. Only applies if the `to` place is a coordinate (not a transit
    /// stop). Does not apply to direct connections (see
    /// `directRentalPropulsionTypes`).
    ///
    ///A list of vehicle propulsion types that are allowed to be used from the
    /// last transit stop to the `to` coordinate. If empty (the default),
    /// all propulsion types are allowed. Example: `HUMAN,ELECTRIC,
    /// ELECTRIC_ASSIST`.
    ///
    /// - `post_transit_rental_providers`: Experimental. Expect unannounced
    ///   breaking changes (without version bumps).
    ///
    ///Optional. Only applies if the `to` place is a coordinate (not a transit
    /// stop). Does not apply to direct connections (see
    /// `directRentalProviders`).
    ///
    ///A list of rental providers that are allowed to be used from the last
    /// transit stop to the `to` coordinate. If empty (the default), all
    /// providers are allowed.
    ///
    /// - `pre_transit_modes`: Optional. Default is `WALK`. Only applies if the
    ///   `from` place is a coordinate (not a transit stop). Does not apply to
    ///   direct connections (see `directModes`).
    ///
    ///A list of modes that are allowed to be used from the `from` coordinate
    /// to the first transit stop. Example: `WALK,BIKE_SHARING`.
    ///
    /// - `pre_transit_rental_form_factors`: Experimental. Expect unannounced
    ///   breaking changes (without version bumps).
    ///
    ///Optional. Only applies if the `from` place is a coordinate (not a
    /// transit stop). Does not apply to direct connections (see
    /// `directRentalFormFactors`).
    ///
    ///A list of vehicle type form factors that are allowed to be used from the
    /// `from` coordinate to the first transit stop. If empty (the default),
    /// all form factors are allowed. Example: `BICYCLE,SCOOTER_STANDING`.
    ///
    /// - `pre_transit_rental_propulsion_types`: Experimental. Expect
    ///   unannounced breaking changes (without version bumps).
    ///
    ///Optional. Only applies if the `from` place is a coordinate (not a
    /// transit stop). Does not apply to direct connections (see
    /// `directRentalPropulsionTypes`).
    ///
    ///A list of vehicle propulsion types that are allowed to be used from the
    /// `from` coordinate to the first transit stop. If empty (the default),
    /// all propulsion types are allowed. Example: `HUMAN,ELECTRIC,
    /// ELECTRIC_ASSIST`.
    ///
    /// - `pre_transit_rental_providers`: Experimental. Expect unannounced
    ///   breaking changes (without version bumps).
    ///
    ///Optional. Only applies if the `from` place is a coordinate (not a
    /// transit stop). Does not apply to direct connections (see
    /// `directRentalProviders`).
    ///
    ///A list of rental providers that are allowed to be used from the `from`
    /// coordinate to the first transit stop. If empty (the default), all
    /// providers are allowed.
    ///
    /// - `require_bike_transport`: Optional. Default is `false`.
    ///
    ///If set to `true`, all used transit trips are required to allow bike
    /// carriage.
    ///
    /// - `require_car_transport`: Optional. Default is `false`.
    ///
    ///If set to `true`, all used transit trips are required to allow car
    /// carriage.
    ///
    /// - `search_window`: Optional. Default is 2 hours which is `7200`.
    ///
    ///The length of the search-window in seconds. Default value two hours.
    ///
    ///  - `arriveBy=true`: number of seconds between the earliest departure
    ///    time and latest departure time
    ///  - `arriveBy=false`: number of seconds between the earliest arrival time
    ///    and the latest arrival time
    ///
    /// - `slow_direct`: Optional. Experimental. Adds overtaken direct public
    ///   transit connections.
    /// - `time`: Optional. Defaults to the current time.
    ///
    ///Departure time ($arriveBy=false) / arrival date ($arriveBy=true),
    ///
    /// - `timeout`: Optional. Query timeout in seconds.
    /// - `timetable_view`: Optional. Default is `true`.
    ///
    ///Search for the best trip options within a time window.
    ///If true two itineraries are considered optimal
    ///if one is better on arrival time (earliest wins)
    ///and the other is better on departure time (latest wins).
    ///In combination with arriveBy this parameter cover the following use
    /// cases:
    ///
    ///`timetable=false` = waiting for the first transit departure/arrival is
    /// considered travel time:
    ///  - `arriveBy=true`: event (e.g. a meeting) starts at 10:00 am, compute
    ///    the best journeys that arrive by that time (maximizes departure time)
    ///  - `arriveBy=false`: event (e.g. a meeting) ends at 11:00 am, compute
    ///    the best journeys that depart after that time
    ///
    ///`timetable=true` = optimize "later departure" + "earlier arrival" and
    /// give all options over a time window:
    ///  - `arriveBy=true`: the time window around `date` and `time` refers to
    ///    the arrival time window
    ///  - `arriveBy=false`: the time window around `date` and `time` refers to
    ///    the departure time window
    ///
    /// - `to_place`: \`latitude,longitude[,level]\` tuple with
    /// - latitude and longitude in degrees
    /// - (optional) level: the OSM level (default: 0)
    ///
    ///OR
    ///
    ///stop id
    ///
    /// - `transfer_time_factor`: Optional. Default is 1.0
    ///
    ///Factor to multiply minimum required transfer times with.
    ///Values smaller than 1.0 are not supported.
    ///
    /// - `transit_modes`: Optional. Default is `TRANSIT` which allows all
    ///   transit modes (no restriction).
    ///Allowed modes for the transit part. If empty, no transit connections
    /// will be computed. For example, this can be used to allow only
    /// `METRO,SUBWAY,TRAM`.
    ///
    /// - `use_routed_transfers`: Optional. Default is `false`.
    ///
    ///Whether to use transfers routed on OpenStreetMap data.
    ///
    /// - `via`: List of via stops to visit (only stop IDs, no coordinates
    ///   allowed for now).
    ///Also see the optional parameter `viaMinimumStay` to set a set a minimum
    /// stay duration for each via stop.
    ///
    /// - `via_minimum_stay`: Optional. If not set, the default is `0,0` - no
    ///   stay required.
    ///
    ///For each `via` stop a minimum stay duration in minutes.
    ///
    ///The value `0` signals that it's allowed to stay in the same trip.
    ///This enables via stays without counting a transfer and can lead
    ///to better connections with less transfers. Transfer connections can
    ///still be found with `viaMinimumStay=0`.
    ///
    /// - `with_fares`: Optional. Experimental. If set to true, the response
    ///   will contain fare information.
    /// - `with_scheduled_skipped_stops`: Optional. Include intermediate stops
    ///   where passengers can not alight/board according to schedule.
    ///```ignore
    /// let response = client.plan()
    ///    .additional_transfer_time(additional_transfer_time)
    ///    .algorithm(algorithm)
    ///    .arrive_by(arrive_by)
    ///    .detailed_transfers(detailed_transfers)
    ///    .direct_modes(direct_modes)
    ///    .direct_rental_form_factors(direct_rental_form_factors)
    ///    .direct_rental_propulsion_types(direct_rental_propulsion_types)
    ///    .direct_rental_providers(direct_rental_providers)
    ///    .elevation_costs(elevation_costs)
    ///    .fastest_direct_factor(fastest_direct_factor)
    ///    .fastest_slow_direct_factor(fastest_slow_direct_factor)
    ///    .from_place(from_place)
    ///    .ignore_direct_rental_return_constraints(ignore_direct_rental_return_constraints)
    ///    .ignore_post_transit_rental_return_constraints(ignore_post_transit_rental_return_constraints)
    ///    .ignore_pre_transit_rental_return_constraints(ignore_pre_transit_rental_return_constraints)
    ///    .join_interlined_legs(join_interlined_legs)
    ///    .language(language)
    ///    .luggage(luggage)
    ///    .max_direct_time(max_direct_time)
    ///    .max_matching_distance(max_matching_distance)
    ///    .max_post_transit_time(max_post_transit_time)
    ///    .max_pre_transit_time(max_pre_transit_time)
    ///    .max_transfers(max_transfers)
    ///    .max_travel_time(max_travel_time)
    ///    .min_transfer_time(min_transfer_time)
    ///    .num_itineraries(num_itineraries)
    ///    .page_cursor(page_cursor)
    ///    .passengers(passengers)
    ///    .pedestrian_profile(pedestrian_profile)
    ///    .post_transit_modes(post_transit_modes)
    ///    .post_transit_rental_form_factors(post_transit_rental_form_factors)
    ///    .post_transit_rental_propulsion_types(post_transit_rental_propulsion_types)
    ///    .post_transit_rental_providers(post_transit_rental_providers)
    ///    .pre_transit_modes(pre_transit_modes)
    ///    .pre_transit_rental_form_factors(pre_transit_rental_form_factors)
    ///    .pre_transit_rental_propulsion_types(pre_transit_rental_propulsion_types)
    ///    .pre_transit_rental_providers(pre_transit_rental_providers)
    ///    .require_bike_transport(require_bike_transport)
    ///    .require_car_transport(require_car_transport)
    ///    .search_window(search_window)
    ///    .slow_direct(slow_direct)
    ///    .time(time)
    ///    .timeout(timeout)
    ///    .timetable_view(timetable_view)
    ///    .to_place(to_place)
    ///    .transfer_time_factor(transfer_time_factor)
    ///    .transit_modes(transit_modes)
    ///    .use_routed_transfers(use_routed_transfers)
    ///    .via(via)
    ///    .via_minimum_stay(via_minimum_stay)
    ///    .with_fares(with_fares)
    ///    .with_scheduled_skipped_stops(with_scheduled_skipped_stops)
    ///    .send()
    ///    .await;
    /// ```
    pub fn plan(&self) -> builder::Plan<'_> {
        builder::Plan::new(self)
    }

    ///Street routing from one to many places or many to one.
    ///The order in the response array corresponds to the order of coordinates
    /// of the \`many\` parameter in the query.
    ///
    ///
    ///Sends a `GET` request to `/api/v1/one-to-many`
    ///
    ///Arguments:
    /// - `arrive_by`: true = many to one
    ///false = one to many
    ///
    /// - `elevation_costs`: Optional. Default is `NONE`.
    ///
    ///Set an elevation cost profile, to penalize routes with incline.
    /// - `NONE`: No additional costs for elevations. This is the default
    ///   behavior
    /// - `LOW`: Add a low cost for increase in elevation and incline along the
    ///   way. This will prefer routes with less ascent, if small detours are
    ///   required.
    /// - `HIGH`: Add a high cost for increase in elevation and incline along
    ///   the way. This will prefer routes with less ascent, if larger detours
    ///   are required.
    ///
    ///As using an elevation costs profile will increase the travel duration,
    ///routing through steep terrain may exceed the maximal allowed duration,
    ///causing a location to appear unreachable.
    ///Increasing the maximum travel time for these segments may resolve this
    /// issue.
    ///
    ///Elevation cost profiles are currently used by following street modes:
    /// - `BIKE`
    ///
    /// - `many`: geo locations as latitude;longitude,latitude;longitude,...
    /// - `max`: maximum travel time in seconds
    /// - `max_matching_distance`: maximum matching distance in meters to match
    ///   geo coordinates to the street network
    /// - `mode`: routing profile to use (currently supported: \`WALK\`,
    ///   \`BIKE\`, \`CAR\`)
    ///
    /// - `one`: geo location as latitude;longitude
    ///```ignore
    /// let response = client.one_to_many()
    ///    .arrive_by(arrive_by)
    ///    .elevation_costs(elevation_costs)
    ///    .many(many)
    ///    .max(max)
    ///    .max_matching_distance(max_matching_distance)
    ///    .mode(mode)
    ///    .one(one)
    ///    .send()
    ///    .await;
    /// ```
    pub fn one_to_many(&self) -> builder::OneToMany<'_> {
        builder::OneToMany::new(self)
    }

    ///Computes all reachable locations from a given stop within a set
    /// duration. Each result entry will contain the fastest travel duration
    /// and the number of connections used.
    ///
    ///
    ///Sends a `GET` request to `/api/v1/one-to-all`
    ///
    ///Arguments:
    /// - `additional_transfer_time`: Optional. Default is 0 minutes.
    ///
    ///Additional transfer time reserved for each transfer in minutes.
    ///
    /// - `arrive_by`: true = all to one,
    ///false = one to all
    ///
    /// - `elevation_costs`: Optional. Default is `NONE`.
    ///
    ///Set an elevation cost profile, to penalize routes with incline.
    /// - `NONE`: No additional costs for elevations. This is the default
    ///   behavior
    /// - `LOW`: Add a low cost for increase in elevation and incline along the
    ///   way. This will prefer routes with less ascent, if small detours are
    ///   required.
    /// - `HIGH`: Add a high cost for increase in elevation and incline along
    ///   the way. This will prefer routes with less ascent, if larger detours
    ///   are required.
    ///
    ///As using an elevation costs profile will increase the travel duration,
    ///routing through steep terrain may exceed the maximal allowed duration,
    ///causing a location to appear unreachable.
    ///Increasing the maximum travel time for these segments may resolve this
    /// issue.
    ///
    ///The profile is used for routing on both the first and last mile.
    ///
    ///Elevation cost profiles are currently used by following street modes:
    /// - `BIKE`
    ///
    /// - `max_matching_distance`: Optional. Default is 25 meters.
    ///
    ///Maximum matching distance in meters to match geo coordinates to the
    /// street network.
    ///
    /// - `max_post_transit_time`: Optional. Default is 15min which is `900`.
    ///  - `arriveBy=true`: Maximum time in seconds for the street leg at `one`
    ///    location.
    ///  - `arriveBy=false`: Currently not used
    ///
    /// - `max_pre_transit_time`: Optional. Default is 15min which is `900`.
    ///  - `arriveBy=true`: Currently not used
    ///  - `arriveBy=false`: Maximum time in seconds for the street leg at `one`
    ///    location.
    ///
    /// - `max_transfers`: The maximum number of allowed transfers (i.e.
    ///   interchanges between transit legs,
    ///pre- and postTransit do not count as transfers).
    ///`maxTransfers=0` searches for direct transit connections without any
    /// transfers. If you want to search only for non-transit connections
    /// (`FOOT`, `CAR`, etc.), send an empty `transitModes` parameter
    /// instead.
    ///
    ///If not provided, the routing uses the server-side default value
    ///which is hardcoded and very high to cover all use cases.
    ///
    ///*Warning*: Use with care. Setting this too low can lead to
    ///optimal (e.g. the fastest) journeys not being found.
    ///If this value is too low to reach the destination at all,
    ///it can lead to slow routing performance.
    ///
    ///In plan endpoints before v3, the behavior is off by one,
    ///i.e. `maxTransfers=0` only returns non-transit connections.
    ///
    /// - `max_travel_time`: The maximum travel time in minutes. Defaults to 90.
    ///   The limit may be increased by the server administrator using
    ///   `onetoall_max_travel_minutes` option in `config.yml`. See
    ///   documentation for details.
    /// - `min_transfer_time`: Optional. Default is 0 minutes.
    ///
    ///Minimum transfer time for each transfer in minutes.
    ///
    /// - `one`: \`latitude,longitude[,level]\` tuple with
    /// - latitude and longitude in degrees
    /// - (optional) level: the OSM level (default: 0)
    ///
    ///OR
    ///
    ///stop id
    ///
    /// - `pedestrian_profile`: Optional. Default is `FOOT`.
    ///
    ///Accessibility profile to use for pedestrian routing in transfers
    ///between transit connections and the first and last mile respectively.
    ///
    /// - `post_transit_modes`: Optional. Default is `WALK`. The behavior
    ///   depends on whether `arriveBy` is set:
    ///  - `arriveBy=true`: Only applies if the `one` place is a coordinate (not
    ///    a transit stop).
    ///  - `arriveBy=false`: Currently not used
    ///
    ///A list of modes that are allowed to be used from the last transit stop
    /// to the `to` coordinate. Example: `WALK,BIKE_SHARING`.
    ///
    /// - `pre_transit_modes`: Optional. Default is `WALK`. The behavior depends
    ///   on whether `arriveBy` is set:
    ///  - `arriveBy=true`: Currently not used
    ///  - `arriveBy=false`: Only applies if the `one` place is a coordinate
    ///    (not a transit stop).
    ///
    ///A list of modes that are allowed to be used from the last transit stop
    /// to the `to` coordinate. Example: `WALK,BIKE_SHARING`.
    ///
    /// - `require_bike_transport`: Optional. Default is `false`.
    ///
    ///If set to `true`, all used transit trips are required to allow bike
    /// carriage.
    ///
    /// - `require_car_transport`: Optional. Default is `false`.
    ///
    ///If set to `true`, all used transit trips are required to allow car
    /// carriage.
    ///
    /// - `time`: Optional. Defaults to the current time.
    ///
    ///Departure time ($arriveBy=false) / arrival date ($arriveBy=true),
    ///
    /// - `transfer_time_factor`: Optional. Default is 1.0
    ///
    ///Factor to multiply minimum required transfer times with.
    ///Values smaller than 1.0 are not supported.
    ///
    /// - `transit_modes`: Optional. Default is `TRANSIT` which allows all
    ///   transit modes (no restriction).
    ///Allowed modes for the transit part. If empty, no transit connections
    /// will be computed. For example, this can be used to allow only
    /// `METRO,SUBWAY,TRAM`.
    ///
    /// - `use_routed_transfers`: Optional. Default is `false`.
    ///
    ///Whether to use transfers routed on OpenStreetMap data.
    ///
    ///```ignore
    /// let response = client.one_to_all()
    ///    .additional_transfer_time(additional_transfer_time)
    ///    .arrive_by(arrive_by)
    ///    .elevation_costs(elevation_costs)
    ///    .max_matching_distance(max_matching_distance)
    ///    .max_post_transit_time(max_post_transit_time)
    ///    .max_pre_transit_time(max_pre_transit_time)
    ///    .max_transfers(max_transfers)
    ///    .max_travel_time(max_travel_time)
    ///    .min_transfer_time(min_transfer_time)
    ///    .one(one)
    ///    .pedestrian_profile(pedestrian_profile)
    ///    .post_transit_modes(post_transit_modes)
    ///    .pre_transit_modes(pre_transit_modes)
    ///    .require_bike_transport(require_bike_transport)
    ///    .require_car_transport(require_car_transport)
    ///    .time(time)
    ///    .transfer_time_factor(transfer_time_factor)
    ///    .transit_modes(transit_modes)
    ///    .use_routed_transfers(use_routed_transfers)
    ///    .send()
    ///    .await;
    /// ```
    pub fn one_to_all(&self) -> builder::OneToAll<'_> {
        builder::OneToAll::new(self)
    }

    ///Translate coordinates to the closest address(es)/places/stops
    ///
    ///Sends a `GET` request to `/api/v1/reverse-geocode`
    ///
    ///Arguments:
    /// - `place`: latitude, longitude in degrees
    /// - `type_`: Optional. Default is all types.
    ///
    ///Only return results of the given type.
    ///For example, this can be used to allow only `ADDRESS` and `STOP`
    /// results.
    ///
    ///```ignore
    /// let response = client.reverse_geocode()
    ///    .place(place)
    ///    .type_(type_)
    ///    .send()
    ///    .await;
    /// ```
    pub fn reverse_geocode(&self) -> builder::ReverseGeocode<'_> {
        builder::ReverseGeocode::new(self)
    }

    ///Autocompletion & geocoding that resolves user input addresses including
    /// coordinates
    ///
    ///Sends a `GET` request to `/api/v1/geocode`
    ///
    ///Arguments:
    /// - `language`: language tags as used in OpenStreetMap
    ///(usually ISO 639-1, or ISO 639-2 if there's no ISO 639-1)
    ///
    /// - `place`: Optional. Used for biasing results towards the coordinate.
    ///
    ///Format: latitude,longitude in degrees
    ///
    /// - `place_bias`: Optional. Used for biasing results towards the
    ///   coordinate. Higher number = higher bias.
    ///
    /// - `text`: the (potentially partially typed) address to resolve
    /// - `type_`: Optional. Default is all types.
    ///
    ///Only return results of the given types.
    ///For example, this can be used to allow only `ADDRESS` and `STOP`
    /// results.
    ///
    ///```ignore
    /// let response = client.geocode()
    ///    .language(language)
    ///    .place(place)
    ///    .place_bias(place_bias)
    ///    .text(text)
    ///    .type_(type_)
    ///    .send()
    ///    .await;
    /// ```
    pub fn geocode(&self) -> builder::Geocode<'_> {
        builder::Geocode::new(self)
    }

    ///Get a trip as itinerary
    ///
    ///Sends a `GET` request to `/api/v4/trip`
    ///
    ///Arguments:
    /// - `join_interlined_legs`: Optional. Default is `true`.
    ///
    ///Controls if a trip with stay-seated transfers is returned:
    /// - `joinInterlinedLegs=false`: as several legs (full information about
    ///   all trip numbers, headsigns, etc.).
    ///  Legs that do not require a transfer (stay-seated transfer) are marked
    /// with `interlineWithPreviousLeg=true`.
    /// - `joinInterlinedLegs=true` (default behavior): as only one joined leg
    ///   containing all stops
    ///
    /// - `language`: language tags as used in OpenStreetMap / GTFS
    ///(usually BCP-47 / ISO 639-1, or ISO 639-2 if there's no ISO 639-1)
    ///
    /// - `trip_id`: trip identifier (e.g. from an itinerary leg or stop event)
    /// - `with_scheduled_skipped_stops`: Optional. Include intermediate stops
    ///   where passengers can not alight/board according to schedule.
    ///```ignore
    /// let response = client.trip()
    ///    .join_interlined_legs(join_interlined_legs)
    ///    .language(language)
    ///    .trip_id(trip_id)
    ///    .with_scheduled_skipped_stops(with_scheduled_skipped_stops)
    ///    .send()
    ///    .await;
    /// ```
    pub fn trip(&self) -> builder::Trip<'_> {
        builder::Trip::new(self)
    }

    ///Get the next N departures or arrivals of a stop sorted by time
    ///
    ///Sends a `GET` request to `/api/v4/stoptimes`
    ///
    ///Arguments:
    /// - `arrive_by`: Optional. Default is `false`.
    ///
    ///  - `arriveBy=true`: the parameters `date` and `time` refer to the
    ///    arrival time
    ///  - `arriveBy=false`: the parameters `date` and `time` refer to the
    ///    departure time
    ///
    /// - `direction`: This parameter will be ignored in case `pageCursor` is
    ///   set.
    ///
    ///Optional. Default is
    ///  - `LATER` for `arriveBy=false`
    ///  - `EARLIER` for `arriveBy=true`
    ///
    ///The response will contain the next `n` arrivals / departures
    ///in case `EARLIER` is selected and the previous `n`
    ///arrivals / departures if `LATER` is selected.
    ///
    /// - `exact_radius`: Optional. Default is `false`.
    ///
    ///If set to `true`, only stations that are phyiscally in the radius are
    /// considered. If set to `false`, additionally to the stations in the
    /// radius, equivalences with the same name and children are considered.
    ///
    /// - `fetch_stops`: Experimental. Expect unannounced breaking changes
    ///   (without version bumps).
    ///
    ///Optional. Default is `false`. If set to `true`, the following stops are
    /// returned for departures and the previous stops are returned for
    /// arrivals.
    ///
    /// - `language`: language tags as used in OpenStreetMap / GTFS
    ///(usually BCP-47 / ISO 639-1, or ISO 639-2 if there's no ISO 639-1)
    ///
    /// - `mode`: Optional. Default is all transit modes.
    ///
    ///Only return arrivals/departures of the given modes.
    ///
    /// - `n`: the number of events
    /// - `page_cursor`: Use the cursor to go to the next "page" of stop times.
    ///Copy the cursor from the last response and keep the original request as
    /// is. This will enable you to search for stop times in the next or
    /// previous time-window.
    ///
    /// - `radius`: Optional. Radius in meters.
    ///
    ///Default is that only stop times of the parent of the stop itself
    ///and all stops with the same name (+ their child stops) are returned.
    ///
    ///If set, all stops at parent stations and their child stops in the
    /// specified radius are returned.
    ///
    /// - `stop_id`: stop id of the stop to retrieve departures/arrivals for
    /// - `time`: Optional. Defaults to the current time.
    ///
    /// - `with_scheduled_skipped_stops`: Optional. Include stoptimes where
    ///   passengers can not alight/board according to schedule.
    ///```ignore
    /// let response = client.stoptimes()
    ///    .arrive_by(arrive_by)
    ///    .direction(direction)
    ///    .exact_radius(exact_radius)
    ///    .fetch_stops(fetch_stops)
    ///    .language(language)
    ///    .mode(mode)
    ///    .n(n)
    ///    .page_cursor(page_cursor)
    ///    .radius(radius)
    ///    .stop_id(stop_id)
    ///    .time(time)
    ///    .with_scheduled_skipped_stops(with_scheduled_skipped_stops)
    ///    .send()
    ///    .await;
    /// ```
    pub fn stoptimes(&self) -> builder::Stoptimes<'_> {
        builder::Stoptimes::new(self)
    }

    ///Given a area frame (box defined by top right and bottom left corner) and
    /// a time frame, it returns all trips and their respective shapes that
    /// operate in this area + time frame. Trips are filtered by zoom level.
    /// On low zoom levels, only long distance trains will be shown while on
    /// high zoom levels, also metros, buses and trams will be returned.
    ///
    ///
    ///Sends a `GET` request to `/api/v4/map/trips`
    ///
    ///Arguments:
    /// - `end_time`: end if the time window
    /// - `max`: latitude,longitude pair of the upper left coordinate
    /// - `min`: latitude,longitude pair of the lower right coordinate
    /// - `start_time`: start of the time window
    /// - `zoom`: current zoom level
    ///```ignore
    /// let response = client.trips()
    ///    .end_time(end_time)
    ///    .max(max)
    ///    .min(min)
    ///    .start_time(start_time)
    ///    .zoom(zoom)
    ///    .send()
    ///    .await;
    /// ```
    pub fn trips(&self) -> builder::Trips<'_> {
        builder::Trips::new(self)
    }

    ///initial location to view the map at after loading based on where public
    /// transport should be visible
    ///
    ///Sends a `GET` request to `/api/v1/map/initial`
    ///
    ///```ignore
    /// let response = client.initial()
    ///    .send()
    ///    .await;
    /// ```
    pub fn initial(&self) -> builder::Initial<'_> {
        builder::Initial::new(self)
    }

    ///Get all stops for a map section
    ///
    ///Sends a `GET` request to `/api/v1/map/stops`
    ///
    ///Arguments:
    /// - `max`: latitude,longitude pair of the upper left coordinate
    /// - `min`: latitude,longitude pair of the lower right coordinate
    ///```ignore
    /// let response = client.stops()
    ///    .max(max)
    ///    .min(min)
    ///    .send()
    ///    .await;
    /// ```
    pub fn stops(&self) -> builder::Stops<'_> {
        builder::Stops::new(self)
    }

    ///Get all available levels for a map section
    ///
    ///Sends a `GET` request to `/api/v1/map/levels`
    ///
    ///Arguments:
    /// - `max`: latitude,longitude pair of the upper left coordinate
    /// - `min`: latitude,longitude pair of the lower right coordinate
    ///```ignore
    /// let response = client.levels()
    ///    .max(max)
    ///    .min(min)
    ///    .send()
    ///    .await;
    /// ```
    pub fn levels(&self) -> builder::Levels<'_> {
        builder::Levels::new(self)
    }

    ///Prints all transfers of a timetable location (track, bus stop, etc.)
    ///
    ///Sends a `GET` request to `/api/debug/transfers`
    ///
    ///Arguments:
    /// - `id`: location id
    ///```ignore
    /// let response = client.transfers()
    ///    .id(id)
    ///    .send()
    ///    .await;
    /// ```
    pub fn transfers(&self) -> builder::Transfers<'_> {
        builder::Transfers::new(self)
    }
}

/// Types for composing operation parameters.
#[allow(clippy::all)]
pub mod builder {
    use super::types;
    #[allow(unused_imports)]
    use super::{
        ByteStream, ClientHooks, ClientInfo, Error, OperationInfo, RequestBuilderExt,
        ResponseValue, encode_path,
    };
    ///Builder for [`Client::plan`]
    ///
    ///[`Client::plan`]: super::Client::plan
    #[derive(Debug, Clone)]
    pub struct Plan<'a> {
        client: &'a super::Client,
        additional_transfer_time: Result<Option<i64>, String>,
        algorithm: Result<Option<types::PlanAlgorithm>, String>,
        arrive_by: Result<Option<bool>, String>,
        detailed_transfers: Result<bool, String>,
        direct_modes: Result<Option<::std::vec::Vec<types::Mode>>, String>,
        direct_rental_form_factors:
            Result<Option<::std::vec::Vec<types::RentalFormFactor>>, String>,
        direct_rental_propulsion_types:
            Result<Option<::std::vec::Vec<types::RentalPropulsionType>>, String>,
        direct_rental_providers: Result<Option<::std::vec::Vec<::std::string::String>>, String>,
        elevation_costs: Result<Option<types::ElevationCosts>, String>,
        fastest_direct_factor: Result<Option<f64>, String>,
        fastest_slow_direct_factor: Result<Option<f64>, String>,
        from_place: Result<::std::string::String, String>,
        ignore_direct_rental_return_constraints: Result<Option<bool>, String>,
        ignore_post_transit_rental_return_constraints: Result<Option<bool>, String>,
        ignore_pre_transit_rental_return_constraints: Result<Option<bool>, String>,
        join_interlined_legs: Result<Option<bool>, String>,
        language: Result<Option<::std::string::String>, String>,
        luggage: Result<Option<::std::num::NonZeroU64>, String>,
        max_direct_time: Result<Option<u64>, String>,
        max_matching_distance: Result<Option<f64>, String>,
        max_post_transit_time: Result<Option<u64>, String>,
        max_pre_transit_time: Result<Option<u64>, String>,
        max_transfers: Result<Option<i64>, String>,
        max_travel_time: Result<Option<i64>, String>,
        min_transfer_time: Result<Option<i64>, String>,
        num_itineraries: Result<Option<i64>, String>,
        page_cursor: Result<Option<::std::string::String>, String>,
        passengers: Result<Option<::std::num::NonZeroU64>, String>,
        pedestrian_profile: Result<Option<types::PedestrianProfile>, String>,
        post_transit_modes: Result<Option<::std::vec::Vec<types::Mode>>, String>,
        post_transit_rental_form_factors:
            Result<Option<::std::vec::Vec<types::RentalFormFactor>>, String>,
        post_transit_rental_propulsion_types:
            Result<Option<::std::vec::Vec<types::RentalPropulsionType>>, String>,
        post_transit_rental_providers:
            Result<Option<::std::vec::Vec<::std::string::String>>, String>,
        pre_transit_modes: Result<Option<::std::vec::Vec<types::Mode>>, String>,
        pre_transit_rental_form_factors:
            Result<Option<::std::vec::Vec<types::RentalFormFactor>>, String>,
        pre_transit_rental_propulsion_types:
            Result<Option<::std::vec::Vec<types::RentalPropulsionType>>, String>,
        pre_transit_rental_providers:
            Result<Option<::std::vec::Vec<::std::string::String>>, String>,
        require_bike_transport: Result<Option<bool>, String>,
        require_car_transport: Result<Option<bool>, String>,
        search_window: Result<Option<u64>, String>,
        slow_direct: Result<Option<bool>, String>,
        time: Result<Option<::chrono::DateTime<::chrono::offset::Utc>>, String>,
        timeout: Result<Option<u64>, String>,
        timetable_view: Result<Option<bool>, String>,
        to_place: Result<::std::string::String, String>,
        transfer_time_factor: Result<Option<f64>, String>,
        transit_modes: Result<Option<::std::vec::Vec<types::Mode>>, String>,
        use_routed_transfers: Result<Option<bool>, String>,
        via: Result<Option<::std::vec::Vec<::std::string::String>>, String>,
        via_minimum_stay: Result<Option<::std::vec::Vec<i64>>, String>,
        with_fares: Result<Option<bool>, String>,
        with_scheduled_skipped_stops: Result<Option<bool>, String>,
    }

    impl<'a> Plan<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                additional_transfer_time: Ok(None),
                algorithm: Ok(None),
                arrive_by: Ok(None),
                detailed_transfers: Err("detailed_transfers was not initialized".to_string()),
                direct_modes: Ok(None),
                direct_rental_form_factors: Ok(None),
                direct_rental_propulsion_types: Ok(None),
                direct_rental_providers: Ok(None),
                elevation_costs: Ok(None),
                fastest_direct_factor: Ok(None),
                fastest_slow_direct_factor: Ok(None),
                from_place: Err("from_place was not initialized".to_string()),
                ignore_direct_rental_return_constraints: Ok(None),
                ignore_post_transit_rental_return_constraints: Ok(None),
                ignore_pre_transit_rental_return_constraints: Ok(None),
                join_interlined_legs: Ok(None),
                language: Ok(None),
                luggage: Ok(None),
                max_direct_time: Ok(None),
                max_matching_distance: Ok(None),
                max_post_transit_time: Ok(None),
                max_pre_transit_time: Ok(None),
                max_transfers: Ok(None),
                max_travel_time: Ok(None),
                min_transfer_time: Ok(None),
                num_itineraries: Ok(None),
                page_cursor: Ok(None),
                passengers: Ok(None),
                pedestrian_profile: Ok(None),
                post_transit_modes: Ok(None),
                post_transit_rental_form_factors: Ok(None),
                post_transit_rental_propulsion_types: Ok(None),
                post_transit_rental_providers: Ok(None),
                pre_transit_modes: Ok(None),
                pre_transit_rental_form_factors: Ok(None),
                pre_transit_rental_propulsion_types: Ok(None),
                pre_transit_rental_providers: Ok(None),
                require_bike_transport: Ok(None),
                require_car_transport: Ok(None),
                search_window: Ok(None),
                slow_direct: Ok(None),
                time: Ok(None),
                timeout: Ok(None),
                timetable_view: Ok(None),
                to_place: Err("to_place was not initialized".to_string()),
                transfer_time_factor: Ok(None),
                transit_modes: Ok(None),
                use_routed_transfers: Ok(None),
                via: Ok(None),
                via_minimum_stay: Ok(None),
                with_fares: Ok(None),
                with_scheduled_skipped_stops: Ok(None),
            }
        }

        pub fn additional_transfer_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.additional_transfer_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i64` for additional_transfer_time failed".to_string());
            self
        }

        pub fn algorithm<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::PlanAlgorithm>,
        {
            self.algorithm = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `PlanAlgorithm` for algorithm failed".to_string());
            self
        }

        pub fn arrive_by<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.arrive_by = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for arrive_by failed".to_string());
            self
        }

        pub fn detailed_transfers<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.detailed_transfers = value
                .try_into()
                .map_err(|_| "conversion to `bool` for detailed_transfers failed".to_string());
            self
        }

        pub fn direct_modes<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::Mode>>,
        {
            self.direct_modes = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: vec :: Vec < Mode >` for direct_modes failed".to_string()
            });
            self
        }

        pub fn direct_rental_form_factors<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::RentalFormFactor>>,
        {
            self . direct_rental_form_factors = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < RentalFormFactor >` for direct_rental_form_factors failed" . to_string ()) ;
            self
        }

        pub fn direct_rental_propulsion_types<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::RentalPropulsionType>>,
        {
            self . direct_rental_propulsion_types = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < RentalPropulsionType >` for direct_rental_propulsion_types failed" . to_string ()) ;
            self
        }

        pub fn direct_rental_providers<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
        {
            self . direct_rental_providers = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < :: std :: string :: String >` for direct_rental_providers failed" . to_string ()) ;
            self
        }

        pub fn elevation_costs<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::ElevationCosts>,
        {
            self.elevation_costs = value.try_into().map(Some).map_err(|_| {
                "conversion to `ElevationCosts` for elevation_costs failed".to_string()
            });
            self
        }

        pub fn fastest_direct_factor<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.fastest_direct_factor = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `f64` for fastest_direct_factor failed".to_string());
            self
        }

        pub fn fastest_slow_direct_factor<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.fastest_slow_direct_factor = value.try_into().map(Some).map_err(|_| {
                "conversion to `f64` for fastest_slow_direct_factor failed".to_string()
            });
            self
        }

        pub fn from_place<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.from_place = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for from_place failed".to_string()
            });
            self
        }

        pub fn ignore_direct_rental_return_constraints<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.ignore_direct_rental_return_constraints =
                value.try_into().map(Some).map_err(|_| {
                    "conversion to `bool` for ignore_direct_rental_return_constraints failed"
                        .to_string()
                });
            self
        }

        pub fn ignore_post_transit_rental_return_constraints<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.ignore_post_transit_rental_return_constraints =
                value.try_into().map(Some).map_err(|_| {
                    "conversion to `bool` for ignore_post_transit_rental_return_constraints failed"
                        .to_string()
                });
            self
        }

        pub fn ignore_pre_transit_rental_return_constraints<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.ignore_pre_transit_rental_return_constraints =
                value.try_into().map(Some).map_err(|_| {
                    "conversion to `bool` for ignore_pre_transit_rental_return_constraints failed"
                        .to_string()
                });
            self
        }

        pub fn join_interlined_legs<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.join_interlined_legs = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for join_interlined_legs failed".to_string());
            self
        }

        pub fn language<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.language = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for language failed".to_string()
            });
            self
        }

        pub fn luggage<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::num::NonZeroU64>,
        {
            self.luggage = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: num :: NonZeroU64` for luggage failed".to_string()
            });
            self
        }

        pub fn max_direct_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<u64>,
        {
            self.max_direct_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `u64` for max_direct_time failed".to_string());
            self
        }

        pub fn max_matching_distance<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.max_matching_distance = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `f64` for max_matching_distance failed".to_string());
            self
        }

        pub fn max_post_transit_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<u64>,
        {
            self.max_post_transit_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `u64` for max_post_transit_time failed".to_string());
            self
        }

        pub fn max_pre_transit_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<u64>,
        {
            self.max_pre_transit_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `u64` for max_pre_transit_time failed".to_string());
            self
        }

        pub fn max_transfers<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.max_transfers = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i64` for max_transfers failed".to_string());
            self
        }

        pub fn max_travel_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.max_travel_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i64` for max_travel_time failed".to_string());
            self
        }

        pub fn min_transfer_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.min_transfer_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i64` for min_transfer_time failed".to_string());
            self
        }

        pub fn num_itineraries<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.num_itineraries = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i64` for num_itineraries failed".to_string());
            self
        }

        pub fn page_cursor<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.page_cursor = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for page_cursor failed".to_string()
            });
            self
        }

        pub fn passengers<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::num::NonZeroU64>,
        {
            self.passengers = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: num :: NonZeroU64` for passengers failed".to_string()
            });
            self
        }

        pub fn pedestrian_profile<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::PedestrianProfile>,
        {
            self.pedestrian_profile = value.try_into().map(Some).map_err(|_| {
                "conversion to `PedestrianProfile` for pedestrian_profile failed".to_string()
            });
            self
        }

        pub fn post_transit_modes<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::Mode>>,
        {
            self.post_transit_modes = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: vec :: Vec < Mode >` for post_transit_modes failed"
                    .to_string()
            });
            self
        }

        pub fn post_transit_rental_form_factors<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::RentalFormFactor>>,
        {
            self . post_transit_rental_form_factors = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < RentalFormFactor >` for post_transit_rental_form_factors failed" . to_string ()) ;
            self
        }

        pub fn post_transit_rental_propulsion_types<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::RentalPropulsionType>>,
        {
            self . post_transit_rental_propulsion_types = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < RentalPropulsionType >` for post_transit_rental_propulsion_types failed" . to_string ()) ;
            self
        }

        pub fn post_transit_rental_providers<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
        {
            self . post_transit_rental_providers = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < :: std :: string :: String >` for post_transit_rental_providers failed" . to_string ()) ;
            self
        }

        pub fn pre_transit_modes<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::Mode>>,
        {
            self.pre_transit_modes = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: vec :: Vec < Mode >` for pre_transit_modes failed"
                    .to_string()
            });
            self
        }

        pub fn pre_transit_rental_form_factors<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::RentalFormFactor>>,
        {
            self . pre_transit_rental_form_factors = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < RentalFormFactor >` for pre_transit_rental_form_factors failed" . to_string ()) ;
            self
        }

        pub fn pre_transit_rental_propulsion_types<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::RentalPropulsionType>>,
        {
            self . pre_transit_rental_propulsion_types = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < RentalPropulsionType >` for pre_transit_rental_propulsion_types failed" . to_string ()) ;
            self
        }

        pub fn pre_transit_rental_providers<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
        {
            self . pre_transit_rental_providers = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < :: std :: string :: String >` for pre_transit_rental_providers failed" . to_string ()) ;
            self
        }

        pub fn require_bike_transport<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.require_bike_transport = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for require_bike_transport failed".to_string());
            self
        }

        pub fn require_car_transport<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.require_car_transport = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for require_car_transport failed".to_string());
            self
        }

        pub fn search_window<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<u64>,
        {
            self.search_window = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `u64` for search_window failed".to_string());
            self
        }

        pub fn slow_direct<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.slow_direct = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for slow_direct failed".to_string());
            self
        }

        pub fn time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
        {
            self . time = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: chrono :: DateTime < :: chrono :: offset :: Utc >` for time failed" . to_string ()) ;
            self
        }

        pub fn timeout<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<u64>,
        {
            self.timeout = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `u64` for timeout failed".to_string());
            self
        }

        pub fn timetable_view<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.timetable_view = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for timetable_view failed".to_string());
            self
        }

        pub fn to_place<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.to_place = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for to_place failed".to_string()
            });
            self
        }

        pub fn transfer_time_factor<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.transfer_time_factor = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `f64` for transfer_time_factor failed".to_string());
            self
        }

        pub fn transit_modes<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::Mode>>,
        {
            self.transit_modes = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: vec :: Vec < Mode >` for transit_modes failed".to_string()
            });
            self
        }

        pub fn use_routed_transfers<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.use_routed_transfers = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for use_routed_transfers failed".to_string());
            self
        }

        pub fn via<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
        {
            self . via = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: std :: vec :: Vec < :: std :: string :: String >` for via failed" . to_string ()) ;
            self
        }

        pub fn via_minimum_stay<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<i64>>,
        {
            self.via_minimum_stay = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: vec :: Vec < i64 >` for via_minimum_stay failed"
                    .to_string()
            });
            self
        }

        pub fn with_fares<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.with_fares = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for with_fares failed".to_string());
            self
        }

        pub fn with_scheduled_skipped_stops<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.with_scheduled_skipped_stops = value.try_into().map(Some).map_err(|_| {
                "conversion to `bool` for with_scheduled_skipped_stops failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/v4/plan`
        pub async fn send(self) -> Result<ResponseValue<types::PlanResponse>, Error<()>> {
            let Self {
                client,
                additional_transfer_time,
                algorithm,
                arrive_by,
                detailed_transfers,
                direct_modes,
                direct_rental_form_factors,
                direct_rental_propulsion_types,
                direct_rental_providers,
                elevation_costs,
                fastest_direct_factor,
                fastest_slow_direct_factor,
                from_place,
                ignore_direct_rental_return_constraints,
                ignore_post_transit_rental_return_constraints,
                ignore_pre_transit_rental_return_constraints,
                join_interlined_legs,
                language,
                luggage,
                max_direct_time,
                max_matching_distance,
                max_post_transit_time,
                max_pre_transit_time,
                max_transfers,
                max_travel_time,
                min_transfer_time,
                num_itineraries,
                page_cursor,
                passengers,
                pedestrian_profile,
                post_transit_modes,
                post_transit_rental_form_factors,
                post_transit_rental_propulsion_types,
                post_transit_rental_providers,
                pre_transit_modes,
                pre_transit_rental_form_factors,
                pre_transit_rental_propulsion_types,
                pre_transit_rental_providers,
                require_bike_transport,
                require_car_transport,
                search_window,
                slow_direct,
                time,
                timeout,
                timetable_view,
                to_place,
                transfer_time_factor,
                transit_modes,
                use_routed_transfers,
                via,
                via_minimum_stay,
                with_fares,
                with_scheduled_skipped_stops,
            } = self;
            let additional_transfer_time =
                additional_transfer_time.map_err(Error::InvalidRequest)?;
            let algorithm = algorithm.map_err(Error::InvalidRequest)?;
            let arrive_by = arrive_by.map_err(Error::InvalidRequest)?;
            let detailed_transfers = detailed_transfers.map_err(Error::InvalidRequest)?;
            let direct_modes = direct_modes.map_err(Error::InvalidRequest)?;
            let direct_rental_form_factors =
                direct_rental_form_factors.map_err(Error::InvalidRequest)?;
            let direct_rental_propulsion_types =
                direct_rental_propulsion_types.map_err(Error::InvalidRequest)?;
            let direct_rental_providers = direct_rental_providers.map_err(Error::InvalidRequest)?;
            let elevation_costs = elevation_costs.map_err(Error::InvalidRequest)?;
            let fastest_direct_factor = fastest_direct_factor.map_err(Error::InvalidRequest)?;
            let fastest_slow_direct_factor =
                fastest_slow_direct_factor.map_err(Error::InvalidRequest)?;
            let from_place = from_place.map_err(Error::InvalidRequest)?;
            let ignore_direct_rental_return_constraints =
                ignore_direct_rental_return_constraints.map_err(Error::InvalidRequest)?;
            let ignore_post_transit_rental_return_constraints =
                ignore_post_transit_rental_return_constraints.map_err(Error::InvalidRequest)?;
            let ignore_pre_transit_rental_return_constraints =
                ignore_pre_transit_rental_return_constraints.map_err(Error::InvalidRequest)?;
            let join_interlined_legs = join_interlined_legs.map_err(Error::InvalidRequest)?;
            let language = language.map_err(Error::InvalidRequest)?;
            let luggage = luggage.map_err(Error::InvalidRequest)?;
            let max_direct_time = max_direct_time.map_err(Error::InvalidRequest)?;
            let max_matching_distance = max_matching_distance.map_err(Error::InvalidRequest)?;
            let max_post_transit_time = max_post_transit_time.map_err(Error::InvalidRequest)?;
            let max_pre_transit_time = max_pre_transit_time.map_err(Error::InvalidRequest)?;
            let max_transfers = max_transfers.map_err(Error::InvalidRequest)?;
            let max_travel_time = max_travel_time.map_err(Error::InvalidRequest)?;
            let min_transfer_time = min_transfer_time.map_err(Error::InvalidRequest)?;
            let num_itineraries = num_itineraries.map_err(Error::InvalidRequest)?;
            let page_cursor = page_cursor.map_err(Error::InvalidRequest)?;
            let passengers = passengers.map_err(Error::InvalidRequest)?;
            let pedestrian_profile = pedestrian_profile.map_err(Error::InvalidRequest)?;
            let post_transit_modes = post_transit_modes.map_err(Error::InvalidRequest)?;
            let post_transit_rental_form_factors =
                post_transit_rental_form_factors.map_err(Error::InvalidRequest)?;
            let post_transit_rental_propulsion_types =
                post_transit_rental_propulsion_types.map_err(Error::InvalidRequest)?;
            let post_transit_rental_providers =
                post_transit_rental_providers.map_err(Error::InvalidRequest)?;
            let pre_transit_modes = pre_transit_modes.map_err(Error::InvalidRequest)?;
            let pre_transit_rental_form_factors =
                pre_transit_rental_form_factors.map_err(Error::InvalidRequest)?;
            let pre_transit_rental_propulsion_types =
                pre_transit_rental_propulsion_types.map_err(Error::InvalidRequest)?;
            let pre_transit_rental_providers =
                pre_transit_rental_providers.map_err(Error::InvalidRequest)?;
            let require_bike_transport = require_bike_transport.map_err(Error::InvalidRequest)?;
            let require_car_transport = require_car_transport.map_err(Error::InvalidRequest)?;
            let search_window = search_window.map_err(Error::InvalidRequest)?;
            let slow_direct = slow_direct.map_err(Error::InvalidRequest)?;
            let time = time.map_err(Error::InvalidRequest)?;
            let timeout = timeout.map_err(Error::InvalidRequest)?;
            let timetable_view = timetable_view.map_err(Error::InvalidRequest)?;
            let to_place = to_place.map_err(Error::InvalidRequest)?;
            let transfer_time_factor = transfer_time_factor.map_err(Error::InvalidRequest)?;
            let transit_modes = transit_modes.map_err(Error::InvalidRequest)?;
            let use_routed_transfers = use_routed_transfers.map_err(Error::InvalidRequest)?;
            let via = via.map_err(Error::InvalidRequest)?;
            let via_minimum_stay = via_minimum_stay.map_err(Error::InvalidRequest)?;
            let with_fares = with_fares.map_err(Error::InvalidRequest)?;
            let with_scheduled_skipped_stops =
                with_scheduled_skipped_stops.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v4/plan", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new(
                    "additionalTransferTime",
                    &additional_transfer_time,
                ))
                .query(&progenitor_client::QueryParam::new("algorithm", &algorithm))
                .query(&progenitor_client::QueryParam::new("arriveBy", &arrive_by))
                .query(&progenitor_client::QueryParam::new(
                    "detailedTransfers",
                    &detailed_transfers,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "directModes",
                    &direct_modes,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "directRentalFormFactors",
                    &direct_rental_form_factors,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "directRentalPropulsionTypes",
                    &direct_rental_propulsion_types,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "directRentalProviders",
                    &direct_rental_providers,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "elevationCosts",
                    &elevation_costs,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "fastestDirectFactor",
                    &fastest_direct_factor,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "fastestSlowDirectFactor",
                    &fastest_slow_direct_factor,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "fromPlace",
                    &from_place,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "ignoreDirectRentalReturnConstraints",
                    &ignore_direct_rental_return_constraints,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "ignorePostTransitRentalReturnConstraints",
                    &ignore_post_transit_rental_return_constraints,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "ignorePreTransitRentalReturnConstraints",
                    &ignore_pre_transit_rental_return_constraints,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "joinInterlinedLegs",
                    &join_interlined_legs,
                ))
                .query(&progenitor_client::QueryParam::new("language", &language))
                .query(&progenitor_client::QueryParam::new("luggage", &luggage))
                .query(&progenitor_client::QueryParam::new(
                    "maxDirectTime",
                    &max_direct_time,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxMatchingDistance",
                    &max_matching_distance,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxPostTransitTime",
                    &max_post_transit_time,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxPreTransitTime",
                    &max_pre_transit_time,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxTransfers",
                    &max_transfers,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxTravelTime",
                    &max_travel_time,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "minTransferTime",
                    &min_transfer_time,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "numItineraries",
                    &num_itineraries,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "pageCursor",
                    &page_cursor,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "passengers",
                    &passengers,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "pedestrianProfile",
                    &pedestrian_profile,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "postTransitModes",
                    &post_transit_modes,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "postTransitRentalFormFactors",
                    &post_transit_rental_form_factors,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "postTransitRentalPropulsionTypes",
                    &post_transit_rental_propulsion_types,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "postTransitRentalProviders",
                    &post_transit_rental_providers,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "preTransitModes",
                    &pre_transit_modes,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "preTransitRentalFormFactors",
                    &pre_transit_rental_form_factors,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "preTransitRentalPropulsionTypes",
                    &pre_transit_rental_propulsion_types,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "preTransitRentalProviders",
                    &pre_transit_rental_providers,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "requireBikeTransport",
                    &require_bike_transport,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "requireCarTransport",
                    &require_car_transport,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "searchWindow",
                    &search_window,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "slowDirect",
                    &slow_direct,
                ))
                .query(&progenitor_client::QueryParam::new("time", &time))
                .query(&progenitor_client::QueryParam::new("timeout", &timeout))
                .query(&progenitor_client::QueryParam::new(
                    "timetableView",
                    &timetable_view,
                ))
                .query(&progenitor_client::QueryParam::new("toPlace", &to_place))
                .query(&progenitor_client::QueryParam::new(
                    "transferTimeFactor",
                    &transfer_time_factor,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "transitModes",
                    &transit_modes,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "useRoutedTransfers",
                    &use_routed_transfers,
                ))
                .query(&progenitor_client::QueryParam::new("via", &via))
                .query(&progenitor_client::QueryParam::new(
                    "viaMinimumStay",
                    &via_minimum_stay,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "withFares",
                    &with_fares,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "withScheduledSkippedStops",
                    &with_scheduled_skipped_stops,
                ))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "plan",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::one_to_many`]
    ///
    ///[`Client::one_to_many`]: super::Client::one_to_many
    #[derive(Debug, Clone)]
    pub struct OneToMany<'a> {
        client: &'a super::Client,
        arrive_by: Result<bool, String>,
        elevation_costs: Result<Option<types::ElevationCosts>, String>,
        many: Result<::std::vec::Vec<::std::string::String>, String>,
        max: Result<f64, String>,
        max_matching_distance: Result<f64, String>,
        mode: Result<types::Mode, String>,
        one: Result<::std::string::String, String>,
    }

    impl<'a> OneToMany<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                arrive_by: Err("arrive_by was not initialized".to_string()),
                elevation_costs: Ok(None),
                many: Err("many was not initialized".to_string()),
                max: Err("max was not initialized".to_string()),
                max_matching_distance: Err("max_matching_distance was not initialized".to_string()),
                mode: Err("mode was not initialized".to_string()),
                one: Err("one was not initialized".to_string()),
            }
        }

        pub fn arrive_by<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.arrive_by = value
                .try_into()
                .map_err(|_| "conversion to `bool` for arrive_by failed".to_string());
            self
        }

        pub fn elevation_costs<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::ElevationCosts>,
        {
            self.elevation_costs = value.try_into().map(Some).map_err(|_| {
                "conversion to `ElevationCosts` for elevation_costs failed".to_string()
            });
            self
        }

        pub fn many<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
        {
            self . many = value . try_into () . map_err (| _ | "conversion to `:: std :: vec :: Vec < :: std :: string :: String >` for many failed" . to_string ()) ;
            self
        }

        pub fn max<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.max = value
                .try_into()
                .map_err(|_| "conversion to `f64` for max failed".to_string());
            self
        }

        pub fn max_matching_distance<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.max_matching_distance = value
                .try_into()
                .map_err(|_| "conversion to `f64` for max_matching_distance failed".to_string());
            self
        }

        pub fn mode<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::Mode>,
        {
            self.mode = value
                .try_into()
                .map_err(|_| "conversion to `Mode` for mode failed".to_string());
            self
        }

        pub fn one<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.one = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for one failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/v1/one-to-many`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<::std::vec::Vec<types::Duration>>, Error<()>> {
            let Self {
                client,
                arrive_by,
                elevation_costs,
                many,
                max,
                max_matching_distance,
                mode,
                one,
            } = self;
            let arrive_by = arrive_by.map_err(Error::InvalidRequest)?;
            let elevation_costs = elevation_costs.map_err(Error::InvalidRequest)?;
            let many = many.map_err(Error::InvalidRequest)?;
            let max = max.map_err(Error::InvalidRequest)?;
            let max_matching_distance = max_matching_distance.map_err(Error::InvalidRequest)?;
            let mode = mode.map_err(Error::InvalidRequest)?;
            let one = one.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v1/one-to-many", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("arriveBy", &arrive_by))
                .query(&progenitor_client::QueryParam::new(
                    "elevationCosts",
                    &elevation_costs,
                ))
                .query(&progenitor_client::QueryParam::new("many", &many))
                .query(&progenitor_client::QueryParam::new("max", &max))
                .query(&progenitor_client::QueryParam::new(
                    "maxMatchingDistance",
                    &max_matching_distance,
                ))
                .query(&progenitor_client::QueryParam::new("mode", &mode))
                .query(&progenitor_client::QueryParam::new("one", &one))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "one_to_many",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::one_to_all`]
    ///
    ///[`Client::one_to_all`]: super::Client::one_to_all
    #[derive(Debug, Clone)]
    pub struct OneToAll<'a> {
        client: &'a super::Client,
        additional_transfer_time: Result<Option<i64>, String>,
        arrive_by: Result<Option<bool>, String>,
        elevation_costs: Result<Option<types::ElevationCosts>, String>,
        max_matching_distance: Result<Option<f64>, String>,
        max_post_transit_time: Result<Option<u64>, String>,
        max_pre_transit_time: Result<Option<u64>, String>,
        max_transfers: Result<Option<i64>, String>,
        max_travel_time: Result<i64, String>,
        min_transfer_time: Result<Option<i64>, String>,
        one: Result<::std::string::String, String>,
        pedestrian_profile: Result<Option<types::PedestrianProfile>, String>,
        post_transit_modes: Result<Option<::std::vec::Vec<types::Mode>>, String>,
        pre_transit_modes: Result<Option<::std::vec::Vec<types::Mode>>, String>,
        require_bike_transport: Result<Option<bool>, String>,
        require_car_transport: Result<Option<bool>, String>,
        time: Result<Option<::chrono::DateTime<::chrono::offset::Utc>>, String>,
        transfer_time_factor: Result<Option<f64>, String>,
        transit_modes: Result<Option<::std::vec::Vec<types::Mode>>, String>,
        use_routed_transfers: Result<Option<bool>, String>,
    }

    impl<'a> OneToAll<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                additional_transfer_time: Ok(None),
                arrive_by: Ok(None),
                elevation_costs: Ok(None),
                max_matching_distance: Ok(None),
                max_post_transit_time: Ok(None),
                max_pre_transit_time: Ok(None),
                max_transfers: Ok(None),
                max_travel_time: Err("max_travel_time was not initialized".to_string()),
                min_transfer_time: Ok(None),
                one: Err("one was not initialized".to_string()),
                pedestrian_profile: Ok(None),
                post_transit_modes: Ok(None),
                pre_transit_modes: Ok(None),
                require_bike_transport: Ok(None),
                require_car_transport: Ok(None),
                time: Ok(None),
                transfer_time_factor: Ok(None),
                transit_modes: Ok(None),
                use_routed_transfers: Ok(None),
            }
        }

        pub fn additional_transfer_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.additional_transfer_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i64` for additional_transfer_time failed".to_string());
            self
        }

        pub fn arrive_by<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.arrive_by = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for arrive_by failed".to_string());
            self
        }

        pub fn elevation_costs<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::ElevationCosts>,
        {
            self.elevation_costs = value.try_into().map(Some).map_err(|_| {
                "conversion to `ElevationCosts` for elevation_costs failed".to_string()
            });
            self
        }

        pub fn max_matching_distance<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.max_matching_distance = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `f64` for max_matching_distance failed".to_string());
            self
        }

        pub fn max_post_transit_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<u64>,
        {
            self.max_post_transit_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `u64` for max_post_transit_time failed".to_string());
            self
        }

        pub fn max_pre_transit_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<u64>,
        {
            self.max_pre_transit_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `u64` for max_pre_transit_time failed".to_string());
            self
        }

        pub fn max_transfers<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.max_transfers = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i64` for max_transfers failed".to_string());
            self
        }

        pub fn max_travel_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.max_travel_time = value
                .try_into()
                .map_err(|_| "conversion to `i64` for max_travel_time failed".to_string());
            self
        }

        pub fn min_transfer_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.min_transfer_time = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i64` for min_transfer_time failed".to_string());
            self
        }

        pub fn one<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.one = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for one failed".to_string()
            });
            self
        }

        pub fn pedestrian_profile<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::PedestrianProfile>,
        {
            self.pedestrian_profile = value.try_into().map(Some).map_err(|_| {
                "conversion to `PedestrianProfile` for pedestrian_profile failed".to_string()
            });
            self
        }

        pub fn post_transit_modes<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::Mode>>,
        {
            self.post_transit_modes = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: vec :: Vec < Mode >` for post_transit_modes failed"
                    .to_string()
            });
            self
        }

        pub fn pre_transit_modes<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::Mode>>,
        {
            self.pre_transit_modes = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: vec :: Vec < Mode >` for pre_transit_modes failed"
                    .to_string()
            });
            self
        }

        pub fn require_bike_transport<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.require_bike_transport = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for require_bike_transport failed".to_string());
            self
        }

        pub fn require_car_transport<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.require_car_transport = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for require_car_transport failed".to_string());
            self
        }

        pub fn time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
        {
            self . time = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: chrono :: DateTime < :: chrono :: offset :: Utc >` for time failed" . to_string ()) ;
            self
        }

        pub fn transfer_time_factor<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.transfer_time_factor = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `f64` for transfer_time_factor failed".to_string());
            self
        }

        pub fn transit_modes<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::Mode>>,
        {
            self.transit_modes = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: vec :: Vec < Mode >` for transit_modes failed".to_string()
            });
            self
        }

        pub fn use_routed_transfers<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.use_routed_transfers = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for use_routed_transfers failed".to_string());
            self
        }

        ///Sends a `GET` request to `/api/v1/one-to-all`
        pub async fn send(self) -> Result<ResponseValue<types::Reachable>, Error<()>> {
            let Self {
                client,
                additional_transfer_time,
                arrive_by,
                elevation_costs,
                max_matching_distance,
                max_post_transit_time,
                max_pre_transit_time,
                max_transfers,
                max_travel_time,
                min_transfer_time,
                one,
                pedestrian_profile,
                post_transit_modes,
                pre_transit_modes,
                require_bike_transport,
                require_car_transport,
                time,
                transfer_time_factor,
                transit_modes,
                use_routed_transfers,
            } = self;
            let additional_transfer_time =
                additional_transfer_time.map_err(Error::InvalidRequest)?;
            let arrive_by = arrive_by.map_err(Error::InvalidRequest)?;
            let elevation_costs = elevation_costs.map_err(Error::InvalidRequest)?;
            let max_matching_distance = max_matching_distance.map_err(Error::InvalidRequest)?;
            let max_post_transit_time = max_post_transit_time.map_err(Error::InvalidRequest)?;
            let max_pre_transit_time = max_pre_transit_time.map_err(Error::InvalidRequest)?;
            let max_transfers = max_transfers.map_err(Error::InvalidRequest)?;
            let max_travel_time = max_travel_time.map_err(Error::InvalidRequest)?;
            let min_transfer_time = min_transfer_time.map_err(Error::InvalidRequest)?;
            let one = one.map_err(Error::InvalidRequest)?;
            let pedestrian_profile = pedestrian_profile.map_err(Error::InvalidRequest)?;
            let post_transit_modes = post_transit_modes.map_err(Error::InvalidRequest)?;
            let pre_transit_modes = pre_transit_modes.map_err(Error::InvalidRequest)?;
            let require_bike_transport = require_bike_transport.map_err(Error::InvalidRequest)?;
            let require_car_transport = require_car_transport.map_err(Error::InvalidRequest)?;
            let time = time.map_err(Error::InvalidRequest)?;
            let transfer_time_factor = transfer_time_factor.map_err(Error::InvalidRequest)?;
            let transit_modes = transit_modes.map_err(Error::InvalidRequest)?;
            let use_routed_transfers = use_routed_transfers.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v1/one-to-all", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new(
                    "additionalTransferTime",
                    &additional_transfer_time,
                ))
                .query(&progenitor_client::QueryParam::new("arriveBy", &arrive_by))
                .query(&progenitor_client::QueryParam::new(
                    "elevationCosts",
                    &elevation_costs,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxMatchingDistance",
                    &max_matching_distance,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxPostTransitTime",
                    &max_post_transit_time,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxPreTransitTime",
                    &max_pre_transit_time,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxTransfers",
                    &max_transfers,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "maxTravelTime",
                    &max_travel_time,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "minTransferTime",
                    &min_transfer_time,
                ))
                .query(&progenitor_client::QueryParam::new("one", &one))
                .query(&progenitor_client::QueryParam::new(
                    "pedestrianProfile",
                    &pedestrian_profile,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "postTransitModes",
                    &post_transit_modes,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "preTransitModes",
                    &pre_transit_modes,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "requireBikeTransport",
                    &require_bike_transport,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "requireCarTransport",
                    &require_car_transport,
                ))
                .query(&progenitor_client::QueryParam::new("time", &time))
                .query(&progenitor_client::QueryParam::new(
                    "transferTimeFactor",
                    &transfer_time_factor,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "transitModes",
                    &transit_modes,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "useRoutedTransfers",
                    &use_routed_transfers,
                ))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "one_to_all",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::reverse_geocode`]
    ///
    ///[`Client::reverse_geocode`]: super::Client::reverse_geocode
    #[derive(Debug, Clone)]
    pub struct ReverseGeocode<'a> {
        client: &'a super::Client,
        place: Result<::std::string::String, String>,
        type_: Result<Option<types::LocationType>, String>,
    }

    impl<'a> ReverseGeocode<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                place: Err("place was not initialized".to_string()),
                type_: Ok(None),
            }
        }

        pub fn place<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.place = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for place failed".to_string()
            });
            self
        }

        pub fn type_<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::LocationType>,
        {
            self.type_ = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `LocationType` for type_ failed".to_string());
            self
        }

        ///Sends a `GET` request to `/api/v1/reverse-geocode`
        pub async fn send(self) -> Result<ResponseValue<::std::vec::Vec<types::Match>>, Error<()>> {
            let Self {
                client,
                place,
                type_,
            } = self;
            let place = place.map_err(Error::InvalidRequest)?;
            let type_ = type_.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v1/reverse-geocode", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("place", &place))
                .query(&progenitor_client::QueryParam::new("type", &type_))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "reverse_geocode",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::geocode`]
    ///
    ///[`Client::geocode`]: super::Client::geocode
    #[derive(Debug, Clone)]
    pub struct Geocode<'a> {
        client: &'a super::Client,
        language: Result<Option<::std::string::String>, String>,
        place: Result<Option<::std::string::String>, String>,
        place_bias: Result<Option<f64>, String>,
        text: Result<::std::string::String, String>,
        type_: Result<Option<types::LocationType>, String>,
    }

    impl<'a> Geocode<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                language: Ok(None),
                place: Ok(None),
                place_bias: Ok(None),
                text: Err("text was not initialized".to_string()),
                type_: Ok(None),
            }
        }

        pub fn language<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.language = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for language failed".to_string()
            });
            self
        }

        pub fn place<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.place = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for place failed".to_string()
            });
            self
        }

        pub fn place_bias<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.place_bias = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `f64` for place_bias failed".to_string());
            self
        }

        pub fn text<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.text = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for text failed".to_string()
            });
            self
        }

        pub fn type_<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::LocationType>,
        {
            self.type_ = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `LocationType` for type_ failed".to_string());
            self
        }

        ///Sends a `GET` request to `/api/v1/geocode`
        pub async fn send(self) -> Result<ResponseValue<::std::vec::Vec<types::Match>>, Error<()>> {
            let Self {
                client,
                language,
                place,
                place_bias,
                text,
                type_,
            } = self;
            let language = language.map_err(Error::InvalidRequest)?;
            let place = place.map_err(Error::InvalidRequest)?;
            let place_bias = place_bias.map_err(Error::InvalidRequest)?;
            let text = text.map_err(Error::InvalidRequest)?;
            let type_ = type_.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v1/geocode", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("language", &language))
                .query(&progenitor_client::QueryParam::new("place", &place))
                .query(&progenitor_client::QueryParam::new(
                    "placeBias",
                    &place_bias,
                ))
                .query(&progenitor_client::QueryParam::new("text", &text))
                .query(&progenitor_client::QueryParam::new("type", &type_))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "geocode",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::trip`]
    ///
    ///[`Client::trip`]: super::Client::trip
    #[derive(Debug, Clone)]
    pub struct Trip<'a> {
        client: &'a super::Client,
        join_interlined_legs: Result<Option<bool>, String>,
        language: Result<Option<::std::string::String>, String>,
        trip_id: Result<::std::string::String, String>,
        with_scheduled_skipped_stops: Result<Option<bool>, String>,
    }

    impl<'a> Trip<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                join_interlined_legs: Ok(None),
                language: Ok(None),
                trip_id: Err("trip_id was not initialized".to_string()),
                with_scheduled_skipped_stops: Ok(None),
            }
        }

        pub fn join_interlined_legs<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.join_interlined_legs = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for join_interlined_legs failed".to_string());
            self
        }

        pub fn language<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.language = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for language failed".to_string()
            });
            self
        }

        pub fn trip_id<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.trip_id = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for trip_id failed".to_string()
            });
            self
        }

        pub fn with_scheduled_skipped_stops<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.with_scheduled_skipped_stops = value.try_into().map(Some).map_err(|_| {
                "conversion to `bool` for with_scheduled_skipped_stops failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/v4/trip`
        pub async fn send(self) -> Result<ResponseValue<types::Itinerary>, Error<()>> {
            let Self {
                client,
                join_interlined_legs,
                language,
                trip_id,
                with_scheduled_skipped_stops,
            } = self;
            let join_interlined_legs = join_interlined_legs.map_err(Error::InvalidRequest)?;
            let language = language.map_err(Error::InvalidRequest)?;
            let trip_id = trip_id.map_err(Error::InvalidRequest)?;
            let with_scheduled_skipped_stops =
                with_scheduled_skipped_stops.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v4/trip", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new(
                    "joinInterlinedLegs",
                    &join_interlined_legs,
                ))
                .query(&progenitor_client::QueryParam::new("language", &language))
                .query(&progenitor_client::QueryParam::new("tripId", &trip_id))
                .query(&progenitor_client::QueryParam::new(
                    "withScheduledSkippedStops",
                    &with_scheduled_skipped_stops,
                ))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "trip",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::stoptimes`]
    ///
    ///[`Client::stoptimes`]: super::Client::stoptimes
    #[derive(Debug, Clone)]
    pub struct Stoptimes<'a> {
        client: &'a super::Client,
        arrive_by: Result<Option<bool>, String>,
        direction: Result<Option<types::StoptimesDirection>, String>,
        exact_radius: Result<Option<bool>, String>,
        fetch_stops: Result<Option<bool>, String>,
        language: Result<Option<::std::string::String>, String>,
        mode: Result<Option<::std::vec::Vec<types::Mode>>, String>,
        n: Result<i64, String>,
        page_cursor: Result<Option<::std::string::String>, String>,
        radius: Result<Option<i64>, String>,
        stop_id: Result<::std::string::String, String>,
        time: Result<Option<::chrono::DateTime<::chrono::offset::Utc>>, String>,
        with_scheduled_skipped_stops: Result<Option<bool>, String>,
    }

    impl<'a> Stoptimes<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                arrive_by: Ok(None),
                direction: Ok(None),
                exact_radius: Ok(None),
                fetch_stops: Ok(None),
                language: Ok(None),
                mode: Ok(None),
                n: Err("n was not initialized".to_string()),
                page_cursor: Ok(None),
                radius: Ok(None),
                stop_id: Err("stop_id was not initialized".to_string()),
                time: Ok(None),
                with_scheduled_skipped_stops: Ok(None),
            }
        }

        pub fn arrive_by<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.arrive_by = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for arrive_by failed".to_string());
            self
        }

        pub fn direction<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::StoptimesDirection>,
        {
            self.direction = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `StoptimesDirection` for direction failed".to_string());
            self
        }

        pub fn exact_radius<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.exact_radius = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for exact_radius failed".to_string());
            self
        }

        pub fn fetch_stops<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.fetch_stops = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `bool` for fetch_stops failed".to_string());
            self
        }

        pub fn language<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.language = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for language failed".to_string()
            });
            self
        }

        pub fn mode<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::vec::Vec<types::Mode>>,
        {
            self.mode = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: vec :: Vec < Mode >` for mode failed".to_string()
            });
            self
        }

        pub fn n<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.n = value
                .try_into()
                .map_err(|_| "conversion to `i64` for n failed".to_string());
            self
        }

        pub fn page_cursor<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.page_cursor = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for page_cursor failed".to_string()
            });
            self
        }

        pub fn radius<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<i64>,
        {
            self.radius = value
                .try_into()
                .map(Some)
                .map_err(|_| "conversion to `i64` for radius failed".to_string());
            self
        }

        pub fn stop_id<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.stop_id = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for stop_id failed".to_string()
            });
            self
        }

        pub fn time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
        {
            self . time = value . try_into () . map (Some) . map_err (| _ | "conversion to `:: chrono :: DateTime < :: chrono :: offset :: Utc >` for time failed" . to_string ()) ;
            self
        }

        pub fn with_scheduled_skipped_stops<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<bool>,
        {
            self.with_scheduled_skipped_stops = value.try_into().map(Some).map_err(|_| {
                "conversion to `bool` for with_scheduled_skipped_stops failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/v4/stoptimes`
        pub async fn send(self) -> Result<ResponseValue<types::StoptimesResponse>, Error<()>> {
            let Self {
                client,
                arrive_by,
                direction,
                exact_radius,
                fetch_stops,
                language,
                mode,
                n,
                page_cursor,
                radius,
                stop_id,
                time,
                with_scheduled_skipped_stops,
            } = self;
            let arrive_by = arrive_by.map_err(Error::InvalidRequest)?;
            let direction = direction.map_err(Error::InvalidRequest)?;
            let exact_radius = exact_radius.map_err(Error::InvalidRequest)?;
            let fetch_stops = fetch_stops.map_err(Error::InvalidRequest)?;
            let language = language.map_err(Error::InvalidRequest)?;
            let mode = mode.map_err(Error::InvalidRequest)?;
            let n = n.map_err(Error::InvalidRequest)?;
            let page_cursor = page_cursor.map_err(Error::InvalidRequest)?;
            let radius = radius.map_err(Error::InvalidRequest)?;
            let stop_id = stop_id.map_err(Error::InvalidRequest)?;
            let time = time.map_err(Error::InvalidRequest)?;
            let with_scheduled_skipped_stops =
                with_scheduled_skipped_stops.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v4/stoptimes", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("arriveBy", &arrive_by))
                .query(&progenitor_client::QueryParam::new("direction", &direction))
                .query(&progenitor_client::QueryParam::new(
                    "exactRadius",
                    &exact_radius,
                ))
                .query(&progenitor_client::QueryParam::new(
                    "fetchStops",
                    &fetch_stops,
                ))
                .query(&progenitor_client::QueryParam::new("language", &language))
                .query(&progenitor_client::QueryParam::new("mode", &mode))
                .query(&progenitor_client::QueryParam::new("n", &n))
                .query(&progenitor_client::QueryParam::new(
                    "pageCursor",
                    &page_cursor,
                ))
                .query(&progenitor_client::QueryParam::new("radius", &radius))
                .query(&progenitor_client::QueryParam::new("stopId", &stop_id))
                .query(&progenitor_client::QueryParam::new("time", &time))
                .query(&progenitor_client::QueryParam::new(
                    "withScheduledSkippedStops",
                    &with_scheduled_skipped_stops,
                ))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "stoptimes",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::trips`]
    ///
    ///[`Client::trips`]: super::Client::trips
    #[derive(Debug, Clone)]
    pub struct Trips<'a> {
        client: &'a super::Client,
        end_time: Result<::chrono::DateTime<::chrono::offset::Utc>, String>,
        max: Result<::std::string::String, String>,
        min: Result<::std::string::String, String>,
        start_time: Result<::chrono::DateTime<::chrono::offset::Utc>, String>,
        zoom: Result<f64, String>,
    }

    impl<'a> Trips<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                end_time: Err("end_time was not initialized".to_string()),
                max: Err("max was not initialized".to_string()),
                min: Err("min was not initialized".to_string()),
                start_time: Err("start_time was not initialized".to_string()),
                zoom: Err("zoom was not initialized".to_string()),
            }
        }

        pub fn end_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
        {
            self . end_time = value . try_into () . map_err (| _ | "conversion to `:: chrono :: DateTime < :: chrono :: offset :: Utc >` for end_time failed" . to_string ()) ;
            self
        }

        pub fn max<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.max = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for max failed".to_string()
            });
            self
        }

        pub fn min<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.min = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for min failed".to_string()
            });
            self
        }

        pub fn start_time<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
        {
            self . start_time = value . try_into () . map_err (| _ | "conversion to `:: chrono :: DateTime < :: chrono :: offset :: Utc >` for start_time failed" . to_string ()) ;
            self
        }

        pub fn zoom<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<f64>,
        {
            self.zoom = value
                .try_into()
                .map_err(|_| "conversion to `f64` for zoom failed".to_string());
            self
        }

        ///Sends a `GET` request to `/api/v4/map/trips`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<::std::vec::Vec<types::TripSegment>>, Error<()>> {
            let Self {
                client,
                end_time,
                max,
                min,
                start_time,
                zoom,
            } = self;
            let end_time = end_time.map_err(Error::InvalidRequest)?;
            let max = max.map_err(Error::InvalidRequest)?;
            let min = min.map_err(Error::InvalidRequest)?;
            let start_time = start_time.map_err(Error::InvalidRequest)?;
            let zoom = zoom.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v4/map/trips", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("endTime", &end_time))
                .query(&progenitor_client::QueryParam::new("max", &max))
                .query(&progenitor_client::QueryParam::new("min", &min))
                .query(&progenitor_client::QueryParam::new(
                    "startTime",
                    &start_time,
                ))
                .query(&progenitor_client::QueryParam::new("zoom", &zoom))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "trips",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::initial`]
    ///
    ///[`Client::initial`]: super::Client::initial
    #[derive(Debug, Clone)]
    pub struct Initial<'a> {
        client: &'a super::Client,
    }

    impl<'a> Initial<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/api/v1/map/initial`
        pub async fn send(self) -> Result<ResponseValue<types::InitialResponse>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/api/v1/map/initial", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "initial",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::stops`]
    ///
    ///[`Client::stops`]: super::Client::stops
    #[derive(Debug, Clone)]
    pub struct Stops<'a> {
        client: &'a super::Client,
        max: Result<::std::string::String, String>,
        min: Result<::std::string::String, String>,
    }

    impl<'a> Stops<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                max: Err("max was not initialized".to_string()),
                min: Err("min was not initialized".to_string()),
            }
        }

        pub fn max<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.max = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for max failed".to_string()
            });
            self
        }

        pub fn min<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.min = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for min failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/v1/map/stops`
        pub async fn send(self) -> Result<ResponseValue<::std::vec::Vec<types::Place>>, Error<()>> {
            let Self { client, max, min } = self;
            let max = max.map_err(Error::InvalidRequest)?;
            let min = min.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v1/map/stops", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("max", &max))
                .query(&progenitor_client::QueryParam::new("min", &min))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "stops",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::levels`]
    ///
    ///[`Client::levels`]: super::Client::levels
    #[derive(Debug, Clone)]
    pub struct Levels<'a> {
        client: &'a super::Client,
        max: Result<::std::string::String, String>,
        min: Result<::std::string::String, String>,
    }

    impl<'a> Levels<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                max: Err("max was not initialized".to_string()),
                min: Err("min was not initialized".to_string()),
            }
        }

        pub fn max<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.max = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for max failed".to_string()
            });
            self
        }

        pub fn min<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.min = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for min failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/v1/map/levels`
        pub async fn send(self) -> Result<ResponseValue<::std::vec::Vec<f64>>, Error<()>> {
            let Self { client, max, min } = self;
            let max = max.map_err(Error::InvalidRequest)?;
            let min = min.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/v1/map/levels", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("max", &max))
                .query(&progenitor_client::QueryParam::new("min", &min))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "levels",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`Client::transfers`]
    ///
    ///[`Client::transfers`]: super::Client::transfers
    #[derive(Debug, Clone)]
    pub struct Transfers<'a> {
        client: &'a super::Client,
        id: Result<::std::string::String, String>,
    }

    impl<'a> Transfers<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                id: Err("id was not initialized".to_string()),
            }
        }

        pub fn id<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.id = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for id failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/debug/transfers`
        pub async fn send(self) -> Result<ResponseValue<types::TransfersResponse>, Error<()>> {
            let Self { client, id } = self;
            let id = id.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/debug/transfers", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("id", &id))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "transfers",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }
}

/// Items consumers will typically use such as the Client.
pub mod prelude {
    pub use self::super::Client;
}
