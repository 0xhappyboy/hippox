//! have_head_browser/shared.rs
//! Shared browser management for headful browser automation

use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

static BROWSER: OnceLock<Arc<Mutex<Option<Arc<Browser>>>>> = OnceLock::new();
static CURRENT_TAB: OnceLock<Arc<Mutex<Option<Arc<Tab>>>>> = OnceLock::new();

/// Get or create the global browser instance (headful mode)
pub fn get_or_create_browser() -> Result<Arc<Browser>> {
    let browser_opt = BROWSER.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut browser_guard = browser_opt.lock().unwrap();

    if browser_guard.is_none() {
        let options = LaunchOptions::default_builder()
            .headless(false)
            .window_size(Some((1280, 720)))
            .sandbox(false)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to launch browser: {}", e))?;

        let browser = Browser::new(options)
            .map_err(|e| anyhow::anyhow!("Failed to create browser: {}", e))?;

        *browser_guard = Some(Arc::new(browser));
    }

    browser_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Browser not available"))
        .map(|b| b.clone())
}

/// Get the current tab, create one if doesn't exist
pub fn get_current_tab() -> Result<Arc<Tab>> {
    let _ = get_or_create_browser()?;

    let tab_opt = CURRENT_TAB.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut tab_guard = tab_opt.lock().unwrap();

    if tab_guard.is_none() {
        let browser = get_or_create_browser()?;
        let tab = browser
            .new_tab()
            .map_err(|e| anyhow::anyhow!("Failed to create new tab: {}", e))?;
        *tab_guard = Some(tab);
    }

    tab_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No tab available"))
        .map(|t| t.clone())
}

/// Set the current tab (accepts Arc<Tab>)
pub fn set_current_tab(tab: Arc<Tab>) {
    let tab_opt = CURRENT_TAB.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut tab_guard = tab_opt.lock().unwrap();
    *tab_guard = Some(tab);
}

/// Clear the current tab (when closed)
pub fn clear_current_tab() {
    let tab_opt = CURRENT_TAB.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut tab_guard = tab_opt.lock().unwrap();
    *tab_guard = None;
}

/// Close the browser completely
pub fn close_browser() -> Result<()> {
    let browser_opt = BROWSER.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut browser_guard = browser_opt.lock().unwrap();

    if browser_guard.is_some() {
        *browser_guard = None;
    }

    clear_current_tab();
    Ok(())
}

/// Wait for page to stabilize
pub async fn wait_for_stable(tab: &Tab, wait_ms: u64) {
    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
}
