//! Network drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Network;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "network", feature = "all"))]
    {
        use crate::{
            HttpDownloadDriver,
            drivers::network::{
                // Existing drivers from dns.rs
                BatchPingDriver,
                DirScanDriver,
                DnsBatchLookupDriver,
                DnsLookupDriver,
                DnsTestDriver,
                DnsZoneTransferDriver,
                FirewallCheckDriver,
                FtpDeleteDriver,
                FtpDownloadDriver,
                FtpListDriver,
                FtpUploadDriver,
                HtmlParseDriver,
                HttpRequestDriver,
                HttpUploadDriver,
                IpInfoDriver,
                IpRangeDriver,
                IpValidateDriver,
                LocalIpDriver,
                NetstatDriver,
                NslookupDriver,
                PingDriver,
                PortLookupDriver,
                PortScanDriver,
                PortTestDriver,
                ReadUrlDriver,
                ReverseDnsDriver,
                SensitiveFileScanDriver,
                ServiceDetectDriver,
                SshExecDriver,
                TcpPingDriver,
                TcpReceiveDriver,
                TcpSendDriver,
                WebhookSendDriver,
            },
            udp::{UdpBroadcastDriver, UdpReceiveDriver, UdpSendDriver},
        };
        // HTTP/URL
        map.insert("http_request".to_string(), Arc::new(HttpRequestDriver));
        map.insert("read_url".to_string(), Arc::new(ReadUrlDriver));
        // Ping
        map.insert("ping".to_string(), Arc::new(PingDriver));
        map.insert("tcp_ping".to_string(), Arc::new(TcpPingDriver));
        map.insert("batch_ping".to_string(), Arc::new(BatchPingDriver));
        // DNS
        map.insert("dns_lookup".to_string(), Arc::new(DnsLookupDriver));
        map.insert("reverse_dns".to_string(), Arc::new(ReverseDnsDriver));
        map.insert(
            "dns_batch_lookup".to_string(),
            Arc::new(DnsBatchLookupDriver),
        );
        map.insert("dns_test".to_string(), Arc::new(DnsTestDriver));
        map.insert(
            "dns_zone_transfer".to_string(),
            Arc::new(DnsZoneTransferDriver),
        );
        // IP
        map.insert("ip_info".to_string(), Arc::new(IpInfoDriver));
        map.insert("ip_validate".to_string(), Arc::new(IpValidateDriver));
        map.insert("ip_range".to_string(), Arc::new(IpRangeDriver));
        map.insert("local_ip".to_string(), Arc::new(LocalIpDriver));
        // TCP
        map.insert("tcp_send".to_string(), Arc::new(TcpSendDriver));
        map.insert("tcp_receive".to_string(), Arc::new(TcpReceiveDriver));
        // UDP
        map.insert("udp_send".to_string(), Arc::new(UdpSendDriver));
        map.insert("udp_receive".to_string(), Arc::new(UdpReceiveDriver));
        map.insert("udp_broadcast".to_string(), Arc::new(UdpBroadcastDriver));
        // FTP
        map.insert("ftp_upload".to_string(), Arc::new(FtpUploadDriver));
        map.insert("ftp_download".to_string(), Arc::new(FtpDownloadDriver));
        map.insert("ftp_list".to_string(), Arc::new(FtpListDriver));
        map.insert("ftp_delete".to_string(), Arc::new(FtpDeleteDriver));
        // Port
        map.insert("port_lookup".to_string(), Arc::new(PortLookupDriver));
        map.insert("port_test".to_string(), Arc::new(PortTestDriver));
        // New drivers
        map.insert("port_scan".to_string(), Arc::new(PortScanDriver));
        map.insert("service_detect".to_string(), Arc::new(ServiceDetectDriver));
        map.insert("dir_scan".to_string(), Arc::new(DirScanDriver));
        map.insert(
            "sensitive_file_scan".to_string(),
            Arc::new(SensitiveFileScanDriver),
        );
        map.insert("firewall_check".to_string(), Arc::new(FirewallCheckDriver));
        map.insert("html_parse".to_string(), Arc::new(HtmlParseDriver));
        map.insert("http_download".to_string(), Arc::new(HttpDownloadDriver));
        map.insert("http_upload".to_string(), Arc::new(HttpUploadDriver));
        map.insert("netstat".to_string(), Arc::new(NetstatDriver));
        map.insert("nslookup".to_string(), Arc::new(NslookupDriver));
        map.insert("ssh_exec".to_string(), Arc::new(SshExecDriver));
        map.insert("webhook_send".to_string(), Arc::new(WebhookSendDriver));
    }
}
