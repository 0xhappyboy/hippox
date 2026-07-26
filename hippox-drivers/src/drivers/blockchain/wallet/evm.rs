//! EVM wallet driver
//!
//! This driver provides EVM-compatible blockchain operations including
//! generating wallets, checking balances, sending transactions, and
//! querying token balances.
use crate::DriverCallback;
use crate::DriverCategory;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use ethers::abi::Bytes;
use ethers::providers::Middleware;
use ethers::signers::Signer;
use ethers::types::U256;
use ethers::types::transaction::eip2718::TypedTransaction;
pub use evm_client::{EvmClient, EvmClientError, EvmType};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{debug, info, warn};
/// EVM wallet driver for blockchain operations
#[derive(Debug)]
pub struct EvmWalletDriver;
#[async_trait::async_trait]
impl Driver for EvmWalletDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "blockchain_evm_wallet"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "EVM compatible wallet operations (Ethereum, Arbitrum, BSC, Base, Polygon, Optimism, Avalanche, etc.): generate address, get balance, send transaction, get token balance"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill for EVM-compatible blockchains. Supports 15+ chains including Ethereum, Arbitrum, BSC, Base, HyperEVM, Plasma, Polygon, Optimism, zkSync, StarkNet, Avalanche, Fantom, Ronin, SKALE, Immutable."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "operation".to_string(),
                param_type: "string".to_string(),
                description: "Operation type: generate, balance, send, token_balance, chain_info, health".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("balance".to_string())),
                enum_values: Some(vec![
                    "generate".to_string(),
                    "balance".to_string(),
                    "send".to_string(),
                    "token_balance".to_string(),
                    "chain_info".to_string(),
                    "health".to_string(),
                ]),
            },
            DriverParameter {
                name: "chain".to_string(),
                param_type: "string".to_string(),
                description: "Blockchain: ethereum, arbitrum, bsc, base, hyperevm, plasma, polygon, optimism, zksync, starknet, avalanche, fantom, ronin, skale, immutable".to_string(),
                required: false,
                default: Some(Value::String("ethereum".to_string())),
                example: Some(Value::String("arbitrum".to_string())),
                enum_values: Some(vec![
                    "ethereum".to_string(),
                    "arbitrum".to_string(),
                    "bsc".to_string(),
                    "base".to_string(),
                    "hyperevm".to_string(),
                    "plasma".to_string(),
                    "polygon".to_string(),
                    "optimism".to_string(),
                    "zksync".to_string(),
                    "starknet".to_string(),
                    "avalanche".to_string(),
                    "fantom".to_string(),
                    "ronin".to_string(),
                    "skale".to_string(),
                    "immutable".to_string(),
                ]),
            },
            DriverParameter {
                name: "rpc_url".to_string(),
                param_type: "string".to_string(),
                description: "Custom RPC URL (optional, overrides default chain RPC)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://eth-mainnet.g.alchemy.com/v2/your-key".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "address".to_string(),
                param_type: "string".to_string(),
                description: "EVM address (0x...) for balance check".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "private_key".to_string(),
                param_type: "string".to_string(),
                description: "Private key (hex format) for sending transactions".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0x1234567890abcdef...".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "amount".to_string(),
                param_type: "string".to_string(),
                description: "Amount in native currency (ETH, BNB, etc.) to send".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0.1".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "to_address".to_string(),
                param_type: "string".to_string(),
                description: "Recipient address for send operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "token_address".to_string(),
                param_type: "string".to_string(),
                description: "ERC20 token contract address for token_balance operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string())),
                enum_values: None,
            },
        ]
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "blockchain_evm_wallet",
            "parameters": {
                "operation": "balance",
                "chain": "ethereum",
                "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "{\n  \"address\": \"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0\",\n  \"chain\": \"ethereum\",\n  \"balance_eth\": 1.234,\n  \"balance_wei\": \"1234000000000000000\"\n}".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Blockchain
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing evm_wallet driver");
        let operation = parameters.get("operation").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'operation' parameter");
            DriverError::missing_parameter("operation")
        })?;
        let chain_str = parameters.get("chain").and_then(|v| v.as_str()).unwrap_or("ethereum");
        let rpc_url = parameters.get("rpc_url").and_then(|v| v.as_str());
        debug!("Operation: {}, chain: {}, rpc_url: {:?}", operation, chain_str, rpc_url);
        match operation {
            "generate" => self.generate_wallet(chain_str).await,
            "balance" => {
                let address = parameters.get("address").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing 'address' parameter for balance check");
                    DriverError::missing_parameter("address")
                })?;
                self.get_balance(address, chain_str, rpc_url).await
            }
            "token_balance" => {
                let address = parameters.get("address").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing 'address' parameter");
                    DriverError::missing_parameter("address")
                })?;
                let token_address = parameters.get("token_address").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing 'token_address' parameter");
                    DriverError::missing_parameter("token_address")
                })?;
                self.get_token_balance(address, token_address, chain_str, rpc_url).await
            }
            "send" => {
                let private_key = parameters.get("private_key").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing 'private_key' for send operation");
                    DriverError::missing_parameter("private_key")
                })?;
                let to_address = parameters.get("to_address").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing 'to_address' for send operation");
                    DriverError::missing_parameter("to_address")
                })?;
                let amount = parameters.get("amount").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing 'amount' for send operation");
                    DriverError::missing_parameter("amount")
                })?;
                self.send_transaction(private_key, to_address, amount, chain_str, rpc_url).await
            }
            "chain_info" => self.get_chain_info(chain_str).await,
            "health" => self.check_health(chain_str, rpc_url).await,
            _ => {
                warn!("Unknown operation: {}", operation);
                Err(DriverError::execution(format!("Unknown operation: {}", operation)))
            }
        }
    }
}
impl EvmWalletDriver {
    /// Convert chain string to EvmType
    fn chain_str_to_enum(&self, chain: &str) -> Option<EvmType> {
        debug!("Converting chain string to EvmType: {}", chain);
        let result = match chain {
            "ethereum" => Some(EvmType::ETHEREUM_MAINNET),
            "arbitrum" => Some(EvmType::ARB_MAINNET),
            "bsc" => Some(EvmType::BSC_MAINNET),
            "base" => Some(EvmType::BASE_MAINNET),
            "hyperevm" => Some(EvmType::HYPEREVM_MAINNET),
            "plasma" => Some(EvmType::PLASMA_MAINNET),
            "polygon" => Some(EvmType::POLYGON_MAINNET),
            "optimism" => Some(EvmType::OPTIMISM_MAINNET),
            "zksync" => Some(EvmType::ZKSYNC_MAINNET),
            "starknet" => Some(EvmType::STARKNET_MAINNET),
            "avalanche" => Some(EvmType::AVALANCHE_MAINNET),
            "fantom" => Some(EvmType::FANTOM_MAINNET),
            "ronin" => Some(EvmType::RONIN_MAINNET),
            "skale" => Some(EvmType::SKALE_MAINNET),
            "immutable" => Some(EvmType::IMMUTABLE_MAINNET),
            _ => None,
        };
        debug!("Chain conversion result: {:?}", result);
        result
    }
    /// Get chain name from EvmType
    fn get_chain_name(&self, evm_type: &EvmType) -> &'static str {
        evm_type.name()
    }
    /// Get native currency symbol for a chain
    fn get_native_symbol(&self, chain: &str) -> &str {
        let symbol = match chain {
            "ethereum" => "ETH",
            "arbitrum" => "ETH",
            "bsc" => "BNB",
            "base" => "ETH",
            "hyperevm" => "HYPE",
            "plasma" => "PLASMA",
            "polygon" => "MATIC",
            "optimism" => "ETH",
            "zksync" => "ETH",
            "starknet" => "STRK",
            "avalanche" => "AVAX",
            "fantom" => "FTM",
            "ronin" => "RON",
            "skale" => "SKL",
            "immutable" => "IMX",
            _ => "ETH",
        };
        debug!("Native symbol for {}: {}", chain, symbol);
        symbol
    }
    /// Create an EVM client
    async fn create_client(&self, chain_str: &str, rpc_url: Option<&str>) -> DriverResult<EvmClient> {
        debug!("Creating EVM client for: {}", chain_str);
        if let Some(url) = rpc_url {
            debug!("Using custom RPC URL: {}", url);
            EvmClient::from_rpc(url).await.map_err(|e| DriverError::execution(format!("Failed to connect to custom RPC: {}", e)))
        } else if let Some(evm_type) = self.chain_str_to_enum(chain_str) {
            debug!("Using default RPC for chain");
            EvmClient::from_type(evm_type).await.map_err(|e| DriverError::execution(format!("Failed to connect to {}: {}", chain_str, e)))
        } else {
            warn!("Unknown chain: {}", chain_str);
            Err(DriverError::execution(format!("Unknown chain: {}", chain_str)))
        }
    }
    /// Create an EVM client with wallet
    async fn create_client_with_wallet(&self, chain_str: &str, private_key: &str, rpc_url: Option<&str>) -> DriverResult<EvmClient> {
        debug!("Creating EVM client with wallet for: {}", chain_str);
        if let Some(url) = rpc_url {
            debug!("Using custom RPC URL: {}", url);
            EvmClient::from_rpc_and_wallet(url, private_key)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to create wallet client: {}", e)))
        } else if let Some(evm_type) = self.chain_str_to_enum(chain_str) {
            debug!("Using default RPC for chain");
            EvmClient::from_wallet(evm_type, private_key).await.map_err(|e| DriverError::execution(format!("Failed to create wallet client: {}", e)))
        } else {
            warn!("Unknown chain: {}", chain_str);
            Err(DriverError::execution(format!("Unknown chain: {}", chain_str)))
        }
    }
    /// Generate a new EVM wallet
    async fn generate_wallet(&self, chain: &str) -> DriverResult<String> {
        debug!("Generating EVM wallet on {}", chain);
        let symbol = self.get_native_symbol(chain);
        let wallet_info = json!({
            "address": format!("0x{}", "x".repeat(40)),
            "private_key": format!("0x{}", "y".repeat(64)),
            "public_key": format!("0x{}", "z".repeat(128)),
            "chain": chain,
            "symbol": symbol,
            "note": "This is a simulated wallet. For production, use ethers::signers::LocalWallet::new_random()."
        });
        info!("EVM wallet generated on {}", chain);
        Ok(serde_json::to_string_pretty(&wallet_info).map_err(|e| DriverError::execution(format!("Failed to serialize wallet info: {}", e)))?)
    }
    /// Get balance for an EVM address
    async fn get_balance(&self, address: &str, chain_str: &str, rpc_url: Option<&str>) -> DriverResult<String> {
        debug!("Getting balance for address: {} on {}", address, chain_str);
        let client = self.create_client(chain_str, rpc_url).await?;
        if let Err(e) = client.health().await {
            warn!("Health check failed: {}", e);
            return Ok(serde_json::to_string_pretty(&json!({
                "error": format!("Health check failed: {}", e),
                "address": address,
                "chain": chain_str,
                "simulated": true,
            }))
            .map_err(|e| DriverError::execution(format!("Failed to serialize response: {}", e)))?);
        }
        let address_parsed =
            address.parse::<ethers::types::Address>().map_err(|e| DriverError::validation("address", format!("Invalid address format: {}", e)))?;
        match client.provider.get_balance(address_parsed, None).await {
            Ok(balance_wei) => {
                let balance_wei_u128: u128 = balance_wei.as_u128();
                let balance_native = balance_wei_u128 as f64 / 1e18;
                let symbol = self.get_native_symbol(chain_str);
                let chain_name =
                    if let Some(evm_type) = client.evm_type { self.get_chain_name(&evm_type).to_string() } else { chain_str.to_string() };
                debug!("Balance: {} {} ({} wei)", balance_native, symbol, balance_wei);
                let output = json!({
                    "address": address,
                    "chain": chain_str,
                    "chain_name": chain_name,
                    format!("balance_{}", symbol.to_lowercase()): balance_native,
                    "balance_wei": balance_wei.to_string(),
                    "symbol": symbol,
                });
                info!("Balance retrieved for {}", address);
                Ok(serde_json::to_string_pretty(&output).map_err(|e| DriverError::execution(format!("Failed to serialize balance: {}", e)))?)
            }
            Err(e) => {
                warn!("Failed to get balance: {}", e);
                Err(DriverError::execution(format!("Failed to get balance: {}", e)))
            }
        }
    }
    /// Get token balance for an EVM address
    async fn get_token_balance(&self, address: &str, token_address: &str, chain_str: &str, rpc_url: Option<&str>) -> DriverResult<String> {
        debug!("Getting token balance for address: {} token: {} on {}", address, token_address, chain_str);
        let client = self.create_client(chain_str, rpc_url).await?;
        client.health().await.map_err(|e| DriverError::execution(format!("Health check failed: {}", e)))?;
        let owner =
            address.parse::<ethers::types::Address>().map_err(|e| DriverError::validation("address", format!("Invalid owner address: {}", e)))?;
        let token_contract = token_address
            .parse::<ethers::types::Address>()
            .map_err(|e| DriverError::validation("token_address", format!("Invalid token address: {}", e)))?;
        let balance_abi = ethers::abi::ethabi::Function {
            name: "balanceOf".to_string(),
            inputs: vec![ethers::abi::ethabi::Param {
                name: "owner".to_string(),
                kind: ethers::abi::ethabi::ParamType::Address,
                internal_type: None,
            }],
            outputs: vec![ethers::abi::ethabi::Param { name: "".to_string(), kind: ethers::abi::ethabi::ParamType::Uint(256), internal_type: None }],
            constant: None,
            state_mutability: ethers::abi::ethabi::StateMutability::View,
        };
        let data = balance_abi
            .encode_input(&[ethers::abi::ethabi::Token::Address(owner)])
            .map_err(|e| DriverError::execution(format!("Failed to encode balanceOf call: {}", e)))?;
        let tx = ethers::types::TransactionRequest::new().to(token_contract).data(Bytes::from(data));
        let typed_tx: TypedTransaction = tx.into();
        let call_result = client.provider.call(&typed_tx, None).await;
        match call_result {
            Ok(result_data) => {
                let balance = ethers::abi::ethabi::decode(&[ethers::abi::ethabi::ParamType::Uint(256)], &result_data)
                    .map_err(|e| DriverError::execution(format!("Failed to decode balance: {}", e)))?;
                if let Some(ethers::abi::ethabi::Token::Uint(balance_uint)) = balance.first() {
                    debug!("Token balance raw: {}", balance_uint);
                    let output = json!({
                        "address": address,
                        "token_address": token_address,
                        "chain": chain_str,
                        "balance_raw": balance_uint.to_string(),
                        "note": "Balance in raw units (need to divide by token decimals)",
                    });
                    info!("Token balance retrieved for {}", address);
                    return Ok(serde_json::to_string_pretty(&output)
                        .map_err(|e| DriverError::execution(format!("Failed to serialize token balance: {}", e)))?);
                }
                Err(DriverError::execution("Unexpected return type from balanceOf".to_string()))
            }
            Err(e) => {
                warn!("Failed to get token balance: {}", e);
                Err(DriverError::execution(format!("Failed to get token balance: {}", e)))
            }
        }
    }
    /// Send an EVM transaction
    async fn send_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount: &str,
        chain_str: &str,
        rpc_url: Option<&str>,
    ) -> DriverResult<String> {
        debug!("Sending transaction: {} from private key to {} on {}", amount, to_address, chain_str);
        let client = self.create_client_with_wallet(chain_str, private_key, rpc_url).await?;
        let wallet = client.wallet.as_ref().ok_or_else(|| DriverError::execution("Wallet not initialized".to_string()))?;
        let to = to_address
            .parse::<ethers::types::Address>()
            .map_err(|e| DriverError::validation("to_address", format!("Invalid recipient address: {}", e)))?;
        let amount_f64: f64 = amount.parse().map_err(|e| DriverError::validation("amount", format!("Invalid amount format: {}", e)))?;
        let amount_wei = ethers::types::U256::from((amount_f64 * 1e18) as u64);
        let nonce = client
            .provider
            .get_transaction_count(wallet.address(), None)
            .await
            .map_err(|e| DriverError::execution(format!("Failed to get nonce: {}", e)))?;
        let gas_price = client.provider.get_gas_price().await.map_err(|e| DriverError::execution(format!("Failed to get gas price: {}", e)))?;
        debug!("Transaction details: nonce={}, gas_price={:?}", nonce, gas_price);
        let tx = ethers::types::TransactionRequest::new().to(to).value(amount_wei).nonce(nonce).gas_price(gas_price).gas::<U256>(21000u64.into());
        let pending_tx =
            client.provider.send_transaction(tx, None).await.map_err(|e| DriverError::execution(format!("Failed to send transaction: {}", e)))?;
        let tx_hash = *pending_tx;
        debug!("Transaction sent, tx_hash: {:x}", tx_hash);
        let result = json!({
            "status": "sent",
            "tx_hash": format!("{:x}", tx_hash),
            "from": format!("{:?}", wallet.address()),
            "to": to_address,
            "amount": amount,
            "chain": chain_str,
            "gas_price_gwei": gas_price.as_u128() as f64 / 1e9,
            "nonce": nonce.as_u64(),
            "note": "Transaction sent. Wait for confirmation.",
        });
        info!("Transaction sent: {:x}", tx_hash);
        Ok(serde_json::to_string_pretty(&result).map_err(|e| DriverError::execution(format!("Failed to serialize transaction result: {}", e)))?)
    }
    /// Get chain information
    async fn get_chain_info(&self, chain_str: &str) -> DriverResult<String> {
        debug!("Getting chain info for: {}", chain_str);
        let evm_type = self.chain_str_to_enum(chain_str);
        match evm_type {
            Some(evm_type) => {
                let block_interval = match chain_str {
                    "ethereum" => 12,
                    "arbitrum" => 1,
                    "bsc" => 3,
                    "base" => 2,
                    "polygon" => 2,
                    "optimism" => 2,
                    "avalanche" => 2,
                    "fantom" => 1,
                    _ => 2,
                };
                let rpc_urls: Vec<&str> = evm_type.rpc().iter().take(3).map(|s| *s).collect();
                debug!("Chain info: name={}, chain_id={}", self.get_chain_name(&evm_type), evm_type.chain_id());
                let info = json!({
                    "chain": chain_str,
                    "name": self.get_chain_name(&evm_type),
                    "chain_id": evm_type.chain_id(),
                    "symbol": self.get_native_symbol(chain_str),
                    "block_interval_seconds": block_interval,
                    "rpc_count": evm_type.rpc().len(),
                    "rpc_urls": rpc_urls,
                });
                info!("Chain info retrieved for {}", chain_str);
                Ok(serde_json::to_string_pretty(&info).map_err(|e| DriverError::execution(format!("Failed to serialize chain info: {}", e)))?)
            }
            None => {
                warn!("Unknown chain: {}", chain_str);
                Err(DriverError::execution(format!("Unknown chain: {}", chain_str)))
            }
        }
    }
    /// Check chain health
    async fn check_health(&self, chain_str: &str, rpc_url: Option<&str>) -> DriverResult<String> {
        debug!("Checking health for: {}", chain_str);
        let client_result = self.create_client(chain_str, rpc_url).await;
        match client_result {
            Ok(client) => match client.health().await {
                Ok(_) => {
                    let chain_id = client.provider.get_chainid().await;
                    let block_number = client.provider.get_block_number().await;
                    debug!("Health check passed for {}", chain_str);
                    let result = json!({
                        "status": "healthy",
                        "chain": chain_str,
                        "chain_id": chain_id.ok().map(|id| id.as_u64()),
                        "block_number": block_number.ok().map(|num| num.as_u64()),
                        "connected_via": if rpc_url.is_some() { "custom_rpc" } else { "default_rpc" },
                    });
                    info!("Health check passed for {}", chain_str);
                    Ok(serde_json::to_string_pretty(&result)
                        .map_err(|e| DriverError::execution(format!("Failed to serialize health result: {}", e)))?)
                }
                Err(e) => {
                    warn!("Health check failed for {}: {}", chain_str, e);
                    Ok(serde_json::to_string_pretty(&json!({
                        "status": "unhealthy",
                        "chain": chain_str,
                        "error": format!("{}", e),
                    }))
                    .map_err(|e| DriverError::execution(format!("Failed to serialize error response: {}", e)))?)
                }
            },
            Err(e) => {
                warn!("Failed to create client for {}: {}", chain_str, e);
                Ok(serde_json::to_string_pretty(&json!({
                    "status": "unhealthy",
                    "chain": chain_str,
                    "error": format!("{}", e),
                }))
                .map_err(|e| DriverError::execution(format!("Failed to serialize error response: {}", e)))?)
            }
        }
    }
}
