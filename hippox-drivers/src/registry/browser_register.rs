//! Browser control drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::HaveHeadBrowser;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    {
        use crate::drivers::have_head_browser::*;

        map.insert(
            "have_head_browser_navigate".to_string(),
            Arc::new(HaveHeadBrowserNavigateDriver),
        );
        map.insert(
            "have_head_browser_click".to_string(),
            Arc::new(HaveHeadBrowserClickDriver),
        );
        map.insert(
            "have_head_browser_type".to_string(),
            Arc::new(HaveHeadBrowserTypeDriver),
        );
        map.insert(
            "have_head_browser_get_text".to_string(),
            Arc::new(HaveHeadBrowserGetTextDriver),
        );
        map.insert(
            "have_head_browser_screenshot".to_string(),
            Arc::new(HaveHeadBrowserScreenshotDriver),
        );
        map.insert(
            "have_head_browser_wait".to_string(),
            Arc::new(HaveHeadBrowserWaitDriver),
        );
        map.insert(
            "have_head_browser_execute_js".to_string(),
            Arc::new(HaveHeadBrowserExecuteJsDriver),
        );
        map.insert(
            "have_head_browser_get_url".to_string(),
            Arc::new(HaveHeadBrowserGetUrlDriver),
        );
        map.insert(
            "have_head_browser_get_title".to_string(),
            Arc::new(HaveHeadBrowserGetTitleDriver),
        );
        map.insert(
            "have_head_browser_back".to_string(),
            Arc::new(HaveHeadBrowserBackDriver),
        );
        map.insert(
            "have_head_browser_forward".to_string(),
            Arc::new(HaveHeadBrowserForwardDriver),
        );
        map.insert(
            "have_head_browser_refresh".to_string(),
            Arc::new(HaveHeadBrowserRefreshDriver),
        );
        map.insert(
            "have_head_browser_tab_new".to_string(),
            Arc::new(HaveHeadBrowserTabNewDriver),
        );
        map.insert(
            "have_head_browser_tab_close".to_string(),
            Arc::new(HaveHeadBrowserTabCloseDriver),
        );
        map.insert(
            "have_head_browser_tab_switch".to_string(),
            Arc::new(HaveHeadBrowserTabSwitchDriver),
        );
        map.insert(
            "have_head_browser_find_element".to_string(),
            Arc::new(HaveHeadBrowserFindElementDriver),
        );
        map.insert(
            "have_head_browser_element_exists".to_string(),
            Arc::new(HaveHeadBrowserElementExistsDriver),
        );
        map.insert(
            "have_head_browser_scroll".to_string(),
            Arc::new(HaveHeadBrowserScrollDriver),
        );
        map.insert(
            "have_head_browser_close".to_string(),
            Arc::new(HaveHeadBrowserCloseDriver),
        );
    }
}
