//! Blockchain drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Blockchain;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "blockchain", feature = "all"))]
    {
        use crate::drivers::*;

        map.insert(
            "blockchain_bitcoin_wallet".to_string(),
            Arc::new(BitcoinWalletDriver),
        );
        map.insert(
            "blockchain_evm_wallet".to_string(),
            Arc::new(EvmWalletDriver),
        );
        map.insert(
            "blockchain_solana_wallet".to_string(),
            Arc::new(SolanaWalletDriver),
        );
    }
}
