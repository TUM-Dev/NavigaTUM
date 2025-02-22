use actix_web::{
    App,
    dev::{ServiceFactory, ServiceRequest},
    web,
};
use utoipa_actix_web::UtoipaApp;
use utoipa_redoc::{Redoc, Servable};

#[derive(serde::Serialize, Default)]
#[serde_with::skip_serializing_none]
struct OpenApiLogo {
    /// The URL pointing to the logo.
    ///
    /// MUST be in the format of a URL.
    /// It SHOULD be an absolute URL so your API definition is usable from any location.
    url: String,
    /// background color to use with the logo image.
    ///
    /// MUST be RGB color in hexadecimal format.
    #[serde(rename = "backgroundColor")]
    background_color: Option<String>,
    ///  text to use for the alt HTML tag on the logo image.
    ///
    ///  Defaults to `logo` if nothing is provided.
    #[serde(rename = "altText")]
    alt_text: Option<String>,
    /// URL pointing to the contact page.
    ///
    ///  Defaults to `info.contact.url` field from the API definition.
    href: Option<String>,
}

pub fn add_openapi_docs<T>(app: UtoipaApp<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>,
{
    let (app, mut openapi) = app.split_for_parts();

    add_static_openapi_docs(&mut openapi);
    app.app_data(web::Data::new(openapi.clone()))
        .service(Redoc::with_url("/api", openapi.clone()))
}

fn add_static_openapi_docs(openapi: &mut utoipa::openapi::OpenApi) {
    use utoipa::openapi::extensions::ExtensionsBuilder;
    use utoipa::openapi::external_docs::ExternalDocsBuilder;
    use utoipa::openapi::tag::TagBuilder;
    use utoipa::openapi::{ContactBuilder, InfoBuilder, LicenseBuilder, ServerBuilder};
    let description = r#"Navigating around TUM with excellence – An API to search for rooms,
buildings and other places

NavigaTUM is a tool developed by students for students, to help you get around at [TUM](https://tum.de). Feel free to contribute.

- [x] Interactive/static maps to look up the position of rooms or buildings
- [x] Fast and typo-tolerant search
- [x] Support for different room code formats as well as generic names
- [x] All functionality is also available via an open and well documented API
- [x] Automatically update the data from upstream datasources
- [ ] Allow students/staff to easily submit feedback and data patches
- [ ] Generate maps from CAD data sources
- [ ] Generate turn by turn navigation advice for navigating end to end

If you'd like to help out or join us in this adventure, we would love to talk to you."#;
    openapi.info = InfoBuilder::new()
            .title("NavigaTUM")
            .description(Some(description))
            .terms_of_service(Some("https://nav.tum.de/en/about/privacy"))
            .contact(Some(
                ContactBuilder::new()
                    .name(Some("OpenSource @ TUM e.V."))
                    .url(Some("https://tum.dev/"))
                    .email(Some("navigatum@tum.de"))
                    .build()
            ))
            .license(Some(
                LicenseBuilder::new()
                    .name("GPL v3")
                    .url(Some("https://www.gnu.org/licenses/"))
                    .build()))
            .version(env!("CARGO_PKG_VERSION"))
            .extensions(Some(ExtensionsBuilder::new()
                .add("logo", serde_json::to_value(OpenApiLogo{ 
                    href: Some("https://nav.tum.de".to_string()),
                    url: "https://raw.githubusercontent.com/TUM-Dev/NavigaTUM/refs/heads/main/webclient/app/assets/logos/navigatum.svg".to_string(), 
                    ..OpenApiLogo::default()
                }).unwrap())
                .build()))
            .build();
    openapi.servers = Some(vec![
        ServerBuilder::new()
            .url("https://nav.tum.de")
            .description(Some("production"))
            .build(),
    ]);
    openapi.tags = Some(vec![
        TagBuilder::new()
            .name("locations".to_string())
            .description(Some("API to access/search for location information"))
            .build(),
        TagBuilder::new()
            .name("calendar".to_string())
            .description(Some("APIs to access calendar-data"))
            .build(),
        TagBuilder::new()
            .name("feedback".to_string())
            .description(Some("APIs to give feedback"))
            .build(),
        TagBuilder::new()
            .name("maps".to_string())
            .description(Some("API to access for map-data"))
            .build(),
    ]);
    openapi.external_docs = Some(
        ExternalDocsBuilder::new()
            .url("https://github.com/TUM-Dev/navigatum")
            .description(Some(
                "Visit our GitHub Page for more in-depth documentation",
            ))
            .into(),
    );
    openapi.schema = "http://json-schema.org/draft-07/schema".to_string();
}
