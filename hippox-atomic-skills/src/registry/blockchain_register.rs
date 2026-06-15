//! Blockchain skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Blockchain;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "blockchain", feature = "all"))]
    {
        use crate::skills::*;

        map.insert(
            "blockchain_bitcoin_wallet".to_string(),
            Arc::new(BitcoinWalletSkill),
        );
        map.insert(
            "blockchain_evm_wallet".to_string(),
            Arc::new(EvmWalletSkill),
        );
        map.insert(
            "blockchain_solana_wallet".to_string(),
            Arc::new(SolanaWalletSkill),
        );
    }
}
