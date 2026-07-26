//! have_head_browser/shared.rs
//! Shared browser management for headful browser automation
//!
//! This module provides shared utilities for managing a headful Chrome browser
//! instance with tab management capabilities.
use crate::DriverError;
use crate::result::DriverResult;
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tracing::{debug, info, warn};
/// Global browser instance (headful mode)
static BROWSER: OnceLock<Arc<Mutex<Option<Arc<Browser>>>>> = OnceLock::new();
/// Global current tab instance
static CURRENT_TAB: OnceLock<Arc<Mutex<Option<Arc<Tab>>>>> = OnceLock::new();
/// Get or create the global browser instance (headful mode)
///
/// Launches a new Chrome browser window if one doesn't exist.
/// The browser window is visible to the user.
///
/// # Returns
///
/// * `DriverResult<Arc<Browser>>` - Browser instance or an error
pub fn get_or_create_browser() -> DriverResult<Arc<Browser>> {
    debug!("Getting or creating browser instance");
    let browser_opt = BROWSER.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut browser_guard = browser_opt.lock().unwrap();
    if browser_guard.is_none() {
        info!("Launching new browser window (headful mode)");
        let options = LaunchOptions::default_builder().headless(false).window_size(Some((1280, 720))).sandbox(false).build().map_err(|e| {
            let err_msg = format!("Failed to build launch options: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
        let browser = Browser::new(options).map_err(|e| {
            let err_msg = format!("Failed to create browser: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
        *browser_guard = Some(Arc::new(browser));
        info!("Browser launched successfully");
    }
    let result = browser_guard
        .as_ref()
        .ok_or_else(|| {
            let err_msg = "Browser not available".to_string();
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })
        .map(|b| b.clone());
    debug!("Browser instance retrieved");
    return result;
}
/// Get the current tab, create one if doesn't exist
///
/// Returns the current active tab. If no tab exists, creates a new one.
///
/// # Returns
///
/// * `DriverResult<Arc<Tab>>` - Current tab instance or an error
pub fn get_current_tab() -> DriverResult<Arc<Tab>> {
    debug!("Getting current tab");
    let _ = get_or_create_browser()?;
    let tab_opt = CURRENT_TAB.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut tab_guard = tab_opt.lock().unwrap();
    if tab_guard.is_none() {
        info!("Creating new tab");
        let browser = get_or_create_browser()?;
        let tab = browser.new_tab().map_err(|e| {
            let err_msg = format!("Failed to create new tab: {}", e);
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })?;
        *tab_guard = Some(tab);
        info!("New tab created");
    }
    let result = tab_guard
        .as_ref()
        .ok_or_else(|| {
            let err_msg = "No tab available".to_string();
            warn!("{}", err_msg);
            return DriverError::execution(err_msg);
        })
        .map(|t| t.clone());
    debug!("Current tab retrieved");
    return result;
}
/// Set the current tab (accepts Arc<Tab>)
///
/// Updates the global current tab reference.
///
/// # Arguments
///
/// * `tab` - Arc<Tab> instance to set as current
pub fn set_current_tab(tab: Arc<Tab>) {
    debug!("Setting current tab");
    let tab_opt = CURRENT_TAB.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut tab_guard = tab_opt.lock().unwrap();
    *tab_guard = Some(tab);
    debug!("Current tab set");
}
/// Clear the current tab (when closed)
///
/// Removes the current tab reference when a tab is closed.
pub fn clear_current_tab() {
    debug!("Clearing current tab");
    let tab_opt = CURRENT_TAB.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut tab_guard = tab_opt.lock().unwrap();
    *tab_guard = None;
    debug!("Current tab cleared");
}
/// Close the browser completely
///
/// Closes the browser window and clears all tab references.
///
/// # Returns
///
/// * `DriverResult<()>` - Success or an error
pub fn close_browser() -> DriverResult<()> {
    debug!("Closing browser");
    let browser_opt = BROWSER.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut browser_guard = browser_opt.lock().unwrap();
    if browser_guard.is_some() {
        info!("Browser window closed");
        *browser_guard = None;
    }
    clear_current_tab();
    debug!("Browser closed successfully");
    return Ok(());
}
/// Wait for page to stabilize
///
/// Sleeps for the specified duration to allow the page to stabilize
/// after navigation or interactions.
///
/// # Arguments
///
/// * `tab` - The tab to wait for (unused in current implementation)
/// * `wait_ms` - Milliseconds to wait
pub async fn wait_for_stable(_tab: &Tab, wait_ms: u64) {
    debug!("Waiting for {}ms for page to stabilize", wait_ms);
    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
    debug!("Wait complete");
}
