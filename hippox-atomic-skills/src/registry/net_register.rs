//! Network skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Net;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "net", feature = "all"))]
    {
        use crate::skills::{udp::{UdpBroadcastSkill, UdpReceiveSkill, UdpSendSkill}, *};
        map.insert("net_httprequest".to_string(), Arc::new(HttpRequestSkill));
        map.insert("read_url".to_string(), Arc::new(ReadUrlSkill));
        map.insert("ping".to_string(), Arc::new(PingSkill));
        map.insert("tcp_ping".to_string(), Arc::new(TcpPingSkill));
        map.insert("batch_ping".to_string(), Arc::new(BatchPingSkill));
        map.insert("dns_lookup".to_string(), Arc::new(DnsLookupSkill));
        map.insert("reverse_dns".to_string(), Arc::new(ReverseDnsSkill));
        map.insert(
            "dns_batch_lookup".to_string(),
            Arc::new(DnsBatchLookupSkill),
        );
        map.insert("dns_test".to_string(), Arc::new(DnsTestSkill));
        map.insert("ip_info".to_string(), Arc::new(IpInfoSkill));
        map.insert("ip_validate".to_string(), Arc::new(IpValidateSkill));
        map.insert("ip_range".to_string(), Arc::new(IpRangeSkill));
        map.insert("local_ip".to_string(), Arc::new(LocalIpSkill));
        map.insert("tcp_send".to_string(), Arc::new(TcpSendSkill));
        map.insert("tcp_receive".to_string(), Arc::new(TcpReceiveSkill));
        map.insert("udp_send".to_string(), Arc::new(UdpSendSkill));
        map.insert("udp_receive".to_string(), Arc::new(UdpReceiveSkill));
        map.insert("udp_broadcast".to_string(), Arc::new(UdpBroadcastSkill));
        map.insert("ftp_upload".to_string(), Arc::new(FtpUploadSkill));
        map.insert("ftp_download".to_string(), Arc::new(FtpDownloadSkill));
        map.insert("ftp_list".to_string(), Arc::new(FtpListSkill));
        map.insert("ftp_delete".to_string(), Arc::new(FtpDeleteSkill));
    }
}
