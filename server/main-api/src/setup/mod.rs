#[cfg(not(feature = "skip_db_setup"))]
pub mod database;

#[cfg(not(feature = "skip_ms_setup"))]
pub mod meilisearch;
#[cfg(test)]
pub mod tests;
