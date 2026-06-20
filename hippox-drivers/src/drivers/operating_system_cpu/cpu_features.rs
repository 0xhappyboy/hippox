//! CPU features driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    drivers::operating_system_cpu::common::CpuFeatures,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for detecting CPU features/instructions
#[derive(Debug)]
pub struct CpuFeaturesDriver;

#[async_trait::async_trait]
impl Driver for CpuFeaturesDriver {
    fn name(&self) -> &str {
        "cpu_features"
    }

    fn description(&self) -> &str {
        "Detect CPU instruction set extensions supported by the processor"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check which CPU features are available for optimization"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "cpu_features",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"CPU Features:
✓ SSE
✓ SSE2
✓ SSE3
✓ SSSE3
✓ SSE4.1
✓ SSE4.2
✓ AVX
✓ AVX2
✗ AVX-512
✓ AES-NI
✓ RDRAND
✗ RDSEED
✗ VMX
✗ SVM"#
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemCpu
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let features = detect_features();
        let mut output = String::from("CPU Features:\n");
        output.push_str(&format!("SSE: {}\n", check_mark(features.sse)));
        output.push_str(&format!("SSE2: {}\n", check_mark(features.sse2)));
        output.push_str(&format!("SSE3: {}\n", check_mark(features.sse3)));
        output.push_str(&format!("SSSE3: {}\n", check_mark(features.ssse3)));
        output.push_str(&format!("SSE4.1: {}\n", check_mark(features.sse4_1)));
        output.push_str(&format!("SSE4.2: {}\n", check_mark(features.sse4_2)));
        output.push_str(&format!("AVX: {}\n", check_mark(features.avx)));
        output.push_str(&format!("AVX2: {}\n", check_mark(features.avx2)));
        output.push_str(&format!("AVX-512: {}\n", check_mark(features.avx512f)));
        output.push_str(&format!("AES-NI: {}\n", check_mark(features.aes_ni)));
        output.push_str(&format!("RDRAND: {}\n", check_mark(features.rdrand)));
        output.push_str(&format!("RDSEED: {}\n", check_mark(features.rdseed)));
        output.push_str(&format!(
            "Hypervisor: {}\n",
            check_mark(features.hypervisor)
        ));
        output.push_str(&format!("VMX (Intel): {}\n", check_mark(features.vmx)));
        output.push_str(&format!("SVM (AMD): {}\n", check_mark(features.svm)));
        Ok(output)
    }
}

fn check_mark(flag: bool) -> &'static str {
    if flag { "✓" } else { "✗" }
}

fn detect_features() -> CpuFeatures {
    let mut features = CpuFeatures {
        sse: false,
        sse2: false,
        sse3: false,
        ssse3: false,
        sse4_1: false,
        sse4_2: false,
        avx: false,
        avx2: false,
        avx512f: false,
        aes_ni: false,
        rdrand: false,
        rdseed: false,
        hypervisor: false,
        vmx: false,
        svm: false,
        smx: false,
        intel_pt: false,
        vaes: false,
        vpclmulqdq: false,
    };
    #[cfg(target_arch = "x86_64")]
    {
        use raw_cpuid::CpuId;
        let cpuid = CpuId::new();
        if let Some(feature_info) = cpuid.get_feature_info() {
            features.sse = feature_info.has_sse();
            features.sse2 = feature_info.has_sse2();
            features.sse3 = feature_info.has_sse3();
            features.ssse3 = feature_info.has_ssse3();
            features.sse4_1 = feature_info.has_sse41();
            features.sse4_2 = feature_info.has_sse42();
            features.avx = feature_info.has_avx();
            features.aes_ni = feature_info.has_aesni();
            features.rdrand = feature_info.has_rdrand();
            features.hypervisor = feature_info.has_hypervisor();
            features.vmx = feature_info.has_vmx();
            features.smx = feature_info.has_smx();
        }
        if let Some(ext_features) = cpuid.get_extended_feature_info() {
            features.avx2 = ext_features.has_avx2();
            features.avx512f = ext_features.has_avx512f();
            features.rdseed = ext_features.has_rdseed();
        }
        // Get SVM support from extended CPUID leaf 0x80000001
        if let Some(ext_cpu) = cpuid.get_extended_cpu_topology() {
            // SVM is typically detected via CPUID 0x80000001 ECX bit 2
            // Try to get it from the extended feature info
            if let Some(ext_feat) = cpuid.get_extended_feature_info() {
                // Some versions of raw_cpuid expose SVM here
                // If has_svm() doesn't exist, we skip it
                #[cfg(any())]
                {
                    features.svm = ext_feat.has_svm();
                }
            }
        }
        // Alternative SVM detection via /proc/cpuinfo on Linux
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                if content.lines().any(|line| line.contains("svm")) {
                    features.svm = true;
                }
            }
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // On non-x86 architectures, try to detect via /proc/cpuinfo
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                let flags = content
                    .lines()
                    .find(|line| line.starts_with("flags"))
                    .map(|line| line.to_string())
                    .unwrap_or_default();

                features.sse = flags.contains("sse");
                features.sse2 = flags.contains("sse2");
                features.sse3 = flags.contains("sse3");
                features.ssse3 = flags.contains("ssse3");
                features.sse4_1 = flags.contains("sse4_1") || flags.contains("sse4.1");
                features.sse4_2 = flags.contains("sse4_2") || flags.contains("sse4.2");
                features.avx = flags.contains("avx");
                features.avx2 = flags.contains("avx2");
                features.avx512f = flags.contains("avx512f");
                features.aes_ni = flags.contains("aes");
                features.rdrand = flags.contains("rdrand");
                features.rdseed = flags.contains("rdseed");
                features.hypervisor = flags.contains("hypervisor");
                features.vmx = flags.contains("vmx");
                features.svm = flags.contains("svm");
            }
        }
    }
    features
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_features_metadata() {
        let driver = CpuFeaturesDriver;
        assert_eq!(driver.name(), "cpu_features");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemCpu);
    }
}
