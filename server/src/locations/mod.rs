use actix_web::web;

mod details;
mod nearby;
mod preview;
mod route;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(details::get_handler)
        .service(nearby::nearby_handler)
        .service(preview::maps_handler)
        .service(route::route_handler);
    let tile_cache = std::env::temp_dir().join("tiles");
    if !tile_cache.exists() {
        std::fs::create_dir(tile_cache).unwrap();
    }
}
