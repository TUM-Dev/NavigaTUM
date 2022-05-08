use actix_web::web;

mod get;
mod search;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get::get_handler);
    cfg.service(search::search_handler);
}
