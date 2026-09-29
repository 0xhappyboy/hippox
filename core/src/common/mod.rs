mod http;
use hippox_drivers::{Driver, get_driver_by_name, list_drivers_names};
pub use http::*;
use std::sync::Arc;
pub fn get_driver(name: &str) -> Option<Arc<dyn Driver>> {
    get_driver_by_name(name)
}
pub fn list_drivers() -> Vec<String> {
    list_drivers_names()
}
