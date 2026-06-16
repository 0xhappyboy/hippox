//! Network skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Network;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "network", feature = "all"))]
    {
        use crate::{
            skills::network::{
                // Existing skills from dns.rs
                BatchPingSkill,
                DirScanSkill,
                DnsBatchLookupSkill,
                DnsLookupSkill,
                DnsTestSkill,
                DnsZoneTransferSkill,
                FirewallCheckSkill,
                FtpDeleteSkill,
                FtpDownloadSkill,
                FtpListSkill,
                FtpUploadSkill,
                HttpRequestSkill,
                IpInfoSkill,
                IpRangeSkill,
                IpValidateSkill,
                LocalIpSkill,
                OsFingerprintSkill,
                PacketCaptureSkill,
                PingSkill,
                PortLookupSkill,
                PortScanSkill,
                PortTestSkill,
                ReadUrlSkill,
                ReverseDnsSkill,
                SensitiveFileScanSkill,
                ServiceDetectSkill,
                TcpPingSkill,
                TcpReceiveSkill,
                TcpSendSkill,
                TrafficAnalyzeSkill,
                VulnScanSkill,
            },
            udp::{UdpBroadcastSkill, UdpReceiveSkill, UdpSendSkill},
        };
        // HTTP/URL
        map.insert("http_request".to_string(), Arc::new(HttpRequestSkill));
        map.insert("read_url".to_string(), Arc::new(ReadUrlSkill));
        // Ping
        map.insert("ping".to_string(), Arc::new(PingSkill));
        map.insert("tcp_ping".to_string(), Arc::new(TcpPingSkill));
        map.insert("batch_ping".to_string(), Arc::new(BatchPingSkill));
        // DNS
        map.insert("dns_lookup".to_string(), Arc::new(DnsLookupSkill));
        map.insert("reverse_dns".to_string(), Arc::new(ReverseDnsSkill));
        map.insert(
            "dns_batch_lookup".to_string(),
            Arc::new(DnsBatchLookupSkill),
        );
        map.insert("dns_test".to_string(), Arc::new(DnsTestSkill));
        map.insert(
            "dns_zone_transfer".to_string(),
            Arc::new(DnsZoneTransferSkill),
        );
        // IP
        map.insert("ip_info".to_string(), Arc::new(IpInfoSkill));
        map.insert("ip_validate".to_string(), Arc::new(IpValidateSkill));
        map.insert("ip_range".to_string(), Arc::new(IpRangeSkill));
        map.insert("local_ip".to_string(), Arc::new(LocalIpSkill));
        // TCP
        map.insert("tcp_send".to_string(), Arc::new(TcpSendSkill));
        map.insert("tcp_receive".to_string(), Arc::new(TcpReceiveSkill));
        // UDP
        map.insert("udp_send".to_string(), Arc::new(UdpSendSkill));
        map.insert("udp_receive".to_string(), Arc::new(UdpReceiveSkill));
        map.insert("udp_broadcast".to_string(), Arc::new(UdpBroadcastSkill));
        // FTP
        map.insert("ftp_upload".to_string(), Arc::new(FtpUploadSkill));
        map.insert("ftp_download".to_string(), Arc::new(FtpDownloadSkill));
        map.insert("ftp_list".to_string(), Arc::new(FtpListSkill));
        map.insert("ftp_delete".to_string(), Arc::new(FtpDeleteSkill));
        // Port
        map.insert("port_lookup".to_string(), Arc::new(PortLookupSkill));
        map.insert("port_test".to_string(), Arc::new(PortTestSkill));
        // New skills
        map.insert("port_scan".to_string(), Arc::new(PortScanSkill));
        map.insert("service_detect".to_string(), Arc::new(ServiceDetectSkill));
        map.insert("os_fingerprint".to_string(), Arc::new(OsFingerprintSkill));
        map.insert("dir_scan".to_string(), Arc::new(DirScanSkill));
        map.insert(
            "sensitive_file_scan".to_string(),
            Arc::new(SensitiveFileScanSkill),
        );
        map.insert("firewall_check".to_string(), Arc::new(FirewallCheckSkill));
        map.insert(
            "ip_block".to_string(),
            Arc::new(crate::skills::network::ip_block::IpBlockSkill),
        );
        map.insert("packet_capture".to_string(), Arc::new(PacketCaptureSkill));
        map.insert("traffic_analyze".to_string(), Arc::new(TrafficAnalyzeSkill));
        map.insert("vuln_scan".to_string(), Arc::new(VulnScanSkill));
    }
}
