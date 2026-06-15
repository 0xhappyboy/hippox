//! Browser control skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Browser;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "have_head_browser", feature = "all"))]
    {
        use crate::skills::have_head_browser::*;

        map.insert(
            "have_head_browser_navigate".to_string(),
            Arc::new(HaveHeadBrowserNavigateSkill),
        );
        map.insert(
            "have_head_browser_click".to_string(),
            Arc::new(HaveHeadBrowserClickSkill),
        );
        map.insert(
            "have_head_browser_type".to_string(),
            Arc::new(HaveHeadBrowserTypeSkill),
        );
        map.insert(
            "have_head_browser_get_text".to_string(),
            Arc::new(HaveHeadBrowserGetTextSkill),
        );
        map.insert(
            "have_head_browser_screenshot".to_string(),
            Arc::new(HaveHeadBrowserScreenshotSkill),
        );
        map.insert(
            "have_head_browser_wait".to_string(),
            Arc::new(HaveHeadBrowserWaitSkill),
        );
        map.insert(
            "have_head_browser_execute_js".to_string(),
            Arc::new(HaveHeadBrowserExecuteJsSkill),
        );
        map.insert(
            "have_head_browser_get_url".to_string(),
            Arc::new(HaveHeadBrowserGetUrlSkill),
        );
        map.insert(
            "have_head_browser_get_title".to_string(),
            Arc::new(HaveHeadBrowserGetTitleSkill),
        );
        map.insert(
            "have_head_browser_back".to_string(),
            Arc::new(HaveHeadBrowserBackSkill),
        );
        map.insert(
            "have_head_browser_forward".to_string(),
            Arc::new(HaveHeadBrowserForwardSkill),
        );
        map.insert(
            "have_head_browser_refresh".to_string(),
            Arc::new(HaveHeadBrowserRefreshSkill),
        );
        map.insert(
            "have_head_browser_tab_new".to_string(),
            Arc::new(HaveHeadBrowserTabNewSkill),
        );
        map.insert(
            "have_head_browser_tab_close".to_string(),
            Arc::new(HaveHeadBrowserTabCloseSkill),
        );
        map.insert(
            "have_head_browser_tab_switch".to_string(),
            Arc::new(HaveHeadBrowserTabSwitchSkill),
        );
        map.insert(
            "have_head_browser_find_element".to_string(),
            Arc::new(HaveHeadBrowserFindElementSkill),
        );
        map.insert(
            "have_head_browser_element_exists".to_string(),
            Arc::new(HaveHeadBrowserElementExistsSkill),
        );
        map.insert(
            "have_head_browser_scroll".to_string(),
            Arc::new(HaveHeadBrowserScrollSkill),
        );
        map.insert(
            "have_head_browser_close".to_string(),
            Arc::new(HaveHeadBrowserCloseSkill),
        );
    }
}
