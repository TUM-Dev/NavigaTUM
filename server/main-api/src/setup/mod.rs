
#[cfg(not(feature = "skip_db_setup"))]
pub(crate) mod database;

#[cfg(not(feature = "skip_ms_setup"))]
pub(crate) mod meilisearch;
