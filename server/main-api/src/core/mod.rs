use actix_web::web;

mod get;
mod legacy_redirect;
mod search;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get::get_handler);
    cfg.service(legacy_redirect::legacy_redirect_handler);
    cfg.service(search::search_handler);
}
