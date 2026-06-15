//! WiFi skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Wifi;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "wifi", feature = "all"))]
    {
        use crate::skills::*;
        map.insert("wifi_scan".to_string(), Arc::new(WifiScanSkill));
        map.insert("wifi_list_connections".to_string(), Arc::new(WifiListConnectionsSkill));
        map.insert("wifi_connect".to_string(), Arc::new(WifiConnectSkill));
        map.insert("wifi_disconnect".to_string(), Arc::new(WifiDisconnectSkill));
        map.insert("wifi_status".to_string(), Arc::new(WifiStatusSkill));
        map.insert("wifi_forget".to_string(), Arc::new(WifiForgetSkill));
        map.insert("wifi_turn_on".to_string(), Arc::new(WifiTurnOnSkill));
        map.insert("wifi_turn_off".to_string(), Arc::new(WifiTurnOffSkill));
        map.insert("wifi_hotspot_create".to_string(), Arc::new(WifiHotspotCreateSkill));
        map.insert("wifi_hotspot_stop".to_string(), Arc::new(WifiHotspotStopSkill));
        map.insert("wifi_get_saved_passwords".to_string(), Arc::new(WifiGetSavedPasswordsSkill));
        map.insert("wifi_get_interface_list".to_string(), Arc::new(WifiGetInterfaceListSkill));
        map.insert("wifi_set_interface_power".to_string(), Arc::new(WifiSetInterfacePowerSkill));
        map.insert("wifi_get_channel".to_string(), Arc::new(WifiGetChannelSkill));
        map.insert("wifi_get_noise_level".to_string(), Arc::new(WifiGetNoiseLevelSkill));
        map.insert("wifi_wps_connect".to_string(), Arc::new(WifiWpsConnectSkill));
        map.insert("wifi_analyze_quality".to_string(), Arc::new(WifiAnalyzeQualitySkill));
        map.insert("wifi_connect_hidden".to_string(), Arc::new(WifiConnectHiddenSkill));
        map.insert("wifi_auto_connect_toggle".to_string(), Arc::new(WifiAutoConnectToggleSkill));
        map.insert("wifi_roaming_toggle".to_string(), Arc::new(WifiRoamingToggleSkill));
        map.insert("wifi_ping_gateway".to_string(), Arc::new(WifiPingGatewaySkill));
        map.insert("wifi_band_preference_set".to_string(), Arc::new(WifiBandPreferenceSetSkill));
        map.insert("wifi_band_preference_get".to_string(), Arc::new(WifiBandPreferenceGetSkill));
        map.insert("wifi_mac_address_set".to_string(), Arc::new(WifiMacAddressSetSkill));
        map.insert("wifi_mac_address_get".to_string(), Arc::new(WifiMacAddressGetSkill));
        map.insert("wifi_dns_set".to_string(), Arc::new(WifiDnsSetSkill));
        map.insert("wifi_proxy_set".to_string(), Arc::new(WifiProxySetSkill));
        map.insert("wifi_priority_set".to_string(), Arc::new(WifiPrioritySetSkill));
        map.insert("wifi_export_config".to_string(), Arc::new(WifiExportConfigSkill));
        map.insert("wifi_import_config".to_string(), Arc::new(WifiImportConfigSkill));
    }
}