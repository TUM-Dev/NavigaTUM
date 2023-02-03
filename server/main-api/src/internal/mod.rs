use actix_web::web;

mod list;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list::ids_with_calendar);
}
