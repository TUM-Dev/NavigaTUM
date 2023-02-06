use actix_web::web;

mod get;
mod legacy_redirect;
mod list;
mod search;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get::get_handler);
    cfg.service(legacy_redirect::legacy_redirect_handler);
    cfg.service(search::search_handler);
    cfg.service(list::ids_with_calendar);
}
