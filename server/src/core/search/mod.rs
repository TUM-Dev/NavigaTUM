mod search_executor;

use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::time::Instant;
#[derive(Deserialize)]
pub struct SearchQueryArgs {
    q: String,
    // Limit per facet
    limit_buildings: Option<usize>,
    limit_rooms: Option<usize>,
    limit_all: Option<usize>,
    pre_highlight: Option<String>,
    post_highlight: Option<String>,
}

/// Returned search results by this
#[derive(Serialize, Debug)]
pub struct SearchResults {
    sections: Vec<search_executor::SearchResultsSection>,
    time_ms: u128,
}

#[derive(Deserialize, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SanitisedSearchQueryArgs {
    // Limit per facet
    pub limit_buildings: usize,
    pub limit_rooms: usize,
    pub limit_all: usize,
}

#[get("/search")]
pub async fn search_handler(
    _req: HttpRequest,
    web::Query(args): web::Query<SearchQueryArgs>,
) -> HttpResponse {
    let start_time = Instant::now();

    let (q, highlighting, sanitised_args) = sanitise_args(args);
    let results_sections =
        search_executor::do_geoentry_search(q, highlighting, sanitised_args).await;
    let time_ms = start_time.elapsed().as_millis();

    if results_sections.len() != 2 {
        return HttpResponse::InternalServerError().body("Internal error");
    }
    let search_results = SearchResults {
        sections: results_sections,
        time_ms,
    };
    HttpResponse::Ok().json(search_results)
}

fn sanitise_args(args: SearchQueryArgs) -> (String, (String, String), SanitisedSearchQueryArgs) {
    let sanitised_args = SanitisedSearchQueryArgs {
        limit_buildings: args.limit_buildings.unwrap_or(5).clamp(0, 1_000),
        limit_rooms: args.limit_rooms.unwrap_or(10).clamp(0, 1_000),
        limit_all: args.limit_all.unwrap_or(10).clamp(1, 1_000),
    };
    let highlighting = (
        args.pre_highlight.unwrap_or("<em>".to_string()),
        args.post_highlight.unwrap_or("</em>".to_string()),
    );
    (args.q, highlighting, sanitised_args)
}
