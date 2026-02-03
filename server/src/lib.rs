// Library for navigatum-server exposing modules for use in binaries

pub mod batch_processor;
pub mod db;
pub mod external;
pub mod limited;
pub mod localisation;
pub mod overlays;
pub mod routes;
pub mod search_executor;

// Re-export AppData from main.rs for lib consumers
// Since AppData is defined in main.rs (bin), we need to make it accessible
// For now, consumers should use their own AppData or we move it here
// This is a temporary workaround - ideally AppData should be in a shared module
