//! WiFi drivers registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{DriverCategory, DriverRegistryMap};

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Wifi;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "wifi", feature = "all"))]
    {
        use crate::drivers::*;
        map.insert("wifi_scan".to_string(), Arc::new(WifiScanDriver));
        map.insert("wifi_list_connections".to_string(), Arc::new(WifiListConnectionsDriver));
        map.insert("wifi_connect".to_string(), Arc::new(WifiConnectDriver));
        map.insert("wifi_disconnect".to_string(), Arc::new(WifiDisconnectDriver));
        map.insert("wifi_status".to_string(), Arc::new(WifiStatusDriver));
        map.insert("wifi_forget".to_string(), Arc::new(WifiForgetDriver));
        map.insert("wifi_turn_on".to_string(), Arc::new(WifiTurnOnDriver));
        map.insert("wifi_turn_off".to_string(), Arc::new(WifiTurnOffDriver));
        map.insert("wifi_hotspot_create".to_string(), Arc::new(WifiHotspotCreateDriver));
        map.insert("wifi_hotspot_stop".to_string(), Arc::new(WifiHotspotStopDriver));
        map.insert("wifi_get_saved_passwords".to_string(), Arc::new(WifiGetSavedPasswordsDriver));
        map.insert("wifi_get_interface_list".to_string(), Arc::new(WifiGetInterfaceListDriver));
        map.insert("wifi_set_interface_power".to_string(), Arc::new(WifiSetInterfacePowerDriver));
        map.insert("wifi_get_channel".to_string(), Arc::new(WifiGetChannelDriver));
        map.insert("wifi_get_noise_level".to_string(), Arc::new(WifiGetNoiseLevelDriver));
        map.insert("wifi_wps_connect".to_string(), Arc::new(WifiWpsConnectDriver));
        map.insert("wifi_analyze_quality".to_string(), Arc::new(WifiAnalyzeQualityDriver));
        map.insert("wifi_connect_hidden".to_string(), Arc::new(WifiConnectHiddenDriver));
        map.insert("wifi_auto_connect_toggle".to_string(), Arc::new(WifiAutoConnectToggleDriver));
        map.insert("wifi_roaming_toggle".to_string(), Arc::new(WifiRoamingToggleDriver));
        map.insert("wifi_ping_gateway".to_string(), Arc::new(WifiPingGatewayDriver));
        map.insert("wifi_band_preference_set".to_string(), Arc::new(WifiBandPreferenceSetDriver));
        map.insert("wifi_band_preference_get".to_string(), Arc::new(WifiBandPreferenceGetDriver));
        map.insert("wifi_mac_address_set".to_string(), Arc::new(WifiMacAddressSetDriver));
        map.insert("wifi_mac_address_get".to_string(), Arc::new(WifiMacAddressGetDriver));
        map.insert("wifi_dns_set".to_string(), Arc::new(WifiDnsSetDriver));
        map.insert("wifi_proxy_set".to_string(), Arc::new(WifiProxySetDriver));
        map.insert("wifi_priority_set".to_string(), Arc::new(WifiPrioritySetDriver));
        map.insert("wifi_export_config".to_string(), Arc::new(WifiExportConfigDriver));
        map.insert("wifi_import_config".to_string(), Arc::new(WifiImportConfigDriver));
    }
}