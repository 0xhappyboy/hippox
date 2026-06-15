use crate::executors::types::{Skill, SkillParameter};
use anyhow::Result;
use ethers::abi::Bytes;
use ethers::providers::Middleware;
use ethers::signers::Signer;
use ethers::types::U256;
use ethers::types::transaction::eip2718::TypedTransaction;
pub use evm_client::{EvmClient, EvmClientError, EvmType};
use reqwest::Response;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub struct BitcoinWalletSkill;

#[async_trait::async_trait]
impl Skill for BitcoinWalletSkill {
    fn name(&self) -> &str {
        "blockchain_bitcoin_wallet"
    }

    fn description(&self) -> &str {
        "Bitcoin wallet operations: generate address, get balance, send transaction"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill for Bitcoin blockchain operations. Supports generating new wallets, \
         getting address balances, and sending BTC transactions. Requires network parameter \
         (mainnet/testnet) and private key for sending transactions."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "operation".to_string(),
                param_type: "string".to_string(),
                description: "Operation type: generate, balance, send".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("generate".to_string())),
                enum_values: Some(vec![
                    "generate".to_string(),
                    "balance".to_string(),
                    "send".to_string(),
                ]),
            },
            SkillParameter {
                name: "network".to_string(),
                param_type: "string".to_string(),
                description: "Bitcoin network: mainnet or testnet".to_string(),
                required: false,
                default: Some(Value::String("mainnet".to_string())),
                example: Some(Value::String("testnet".to_string())),
                enum_values: Some(vec!["mainnet".to_string(), "testnet".to_string()]),
            },
            SkillParameter {
                name: "address".to_string(),
                param_type: "string".to_string(),
                description: "Bitcoin address for balance check or send destination".to_string(),
                required: false,
                default: None,
                example: Some(Value::String(
                    "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "private_key".to_string(),
                param_type: "string".to_string(),
                description: "Private key (WIF format) for sending transactions".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("L5oLkpV3...".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "amount".to_string(),
                param_type: "string".to_string(),
                description: "Amount in BTC to send".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0.001".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "to_address".to_string(),
                param_type: "string".to_string(),
                description: "Recipient address for send operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String(
                    "1CounterpartyXXXXXXXXXXXXXXXUWLpV".to_string(),
                )),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "blockchain_bitcoin_wallet",
            "parameters": {
                "operation": "generate",
                "network": "testnet"
            }
        })
    }

    fn example_output(&self) -> String {
        "{\n  \"address\": \"tb1q...\",\n  \"private_key\": \"cV...\",\n  \"public_key\": \"02...\"\n}".to_string()
    }

    fn category(&self) -> &str {
        "blockchain"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let operation = parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'operation' parameter"))?;
        let network = parameters
            .get("network")
            .and_then(|v| v.as_str())
            .unwrap_or("mainnet");
        match operation {
            "generate" => self.generate_wallet(network).await,
            "balance" => {
                let address = parameters
                    .get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing 'address' parameter for balance check")
                    })?;
                self.get_balance(address, network).await
            }
            "send" => {
                let private_key = parameters
                    .get("private_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'private_key' for send operation"))?;
                let to_address = parameters
                    .get("to_address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'to_address' for send operation"))?;
                let amount = parameters
                    .get("amount")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'amount' for send operation"))?;
                self.send_transaction(private_key, to_address, amount, network)
                    .await
            }
            _ => anyhow::bail!("Unknown operation: {}", operation),
        }
    }
}

impl BitcoinWalletSkill {
    async fn generate_wallet(&self, network: &str) -> Result<String> {
        let is_testnet = network == "testnet";
        let prefix = if is_testnet { "tb1" } else { "bc1" };
        let wallet_info = json!({
            "address": format!("{}q{}", prefix, "x".repeat(38)),
            "private_key_wif": format!("L{}", "x".repeat(50)),
            "public_key": format!("02{}", "x".repeat(64)),
            "network": network,
            "note": "This is a simulated wallet. For production, integrate with rust-bitcoin."
        });
        Ok(serde_json::to_string_pretty(&wallet_info)?)
    }

    async fn get_balance(&self, address: &str, network: &str) -> Result<String> {
        let api_url = if network == "testnet" {
            format!(
                "https://blockstream.info/testnet/api/address/{}/utxo",
                address
            )
        } else {
            format!("https://blockstream.info/api/address/{}/utxo", address)
        };
        let client = reqwest::Client::new();
        let response = client
            .get(&api_url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;
        match response {
            Ok(resp) if resp.status().is_success() => {
                let utxos: Value = resp
                    .json()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))?;
                let mut total_satoshis: u64 = 0;
                if let Some(utxo_array) = utxos.as_array() {
                    for utxo in utxo_array {
                        if let Some(value) = utxo.get("value").and_then(|v| v.as_u64()) {
                            total_satoshis += value;
                        }
                    }
                }
                let btc_balance = total_satoshis as f64 / 100_000_000.0;
                let result = json!({
                    "address": address,
                    "network": network,
                    "balance_satoshis": total_satoshis,
                    "balance_btc": btc_balance,
                    "utxo_count": utxos.as_array().map(|a| a.len()).unwrap_or(0),
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
            Ok(resp) => {
                anyhow::bail!("API error: {}", resp.status())
            }
            Err(e) => {
                let simulated = json!({
                    "address": address,
                    "network": network,
                    "balance_satoshis": 0,
                    "balance_btc": 0.0,
                    "note": format!("Could not fetch real balance: {}. Showing simulated balance.", e),
                    "simulated": true,
                });
                Ok(serde_json::to_string_pretty(&simulated)?)
            }
        }
    }

    async fn send_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount_btc: &str,
        network: &str,
    ) -> Result<String> {
        let amount_f64: f64 = amount_btc.parse()?;
        let amount_satoshis = (amount_f64 * 100_000_000.0) as u64;
        let result = json!({
            "status": "simulated",
            "txid": format!("simulated_tx_{}", uuid::Uuid::new_v4()),
            "from": "generated_from_private_key",
            "to": to_address,
            "amount_btc": amount_btc,
            "amount_satoshis": amount_satoshis,
            "network": network,
            "fee_satoshis": 10000,
            "note": "This is a simulated transaction. For real BTC transactions, implement with rust-bitcoin.",
        });
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

#[derive(Debug)]
pub struct EvmWalletSkill;

#[async_trait::async_trait]
impl Skill for EvmWalletSkill {
    fn name(&self) -> &str {
        "blockchain_evm_wallet"
    }

    fn description(&self) -> &str {
        "EVM compatible wallet operations (Ethereum, Arbitrum, BSC, Base, Polygon, Optimism, Avalanche, etc.): generate address, get balance, send transaction, get token balance"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill for EVM-compatible blockchains. Supports 15+ chains including Ethereum, Arbitrum, BSC, Base, HyperEVM, Plasma, Polygon, Optimism, zkSync, StarkNet, Avalanche, Fantom, Ronin, SKALE, Immutable."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
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
            SkillParameter {
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
            SkillParameter {
                name: "rpc_url".to_string(),
                param_type: "string".to_string(),
                description: "Custom RPC URL (optional, overrides default chain RPC)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://eth-mainnet.g.alchemy.com/v2/your-key".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "address".to_string(),
                param_type: "string".to_string(),
                description: "EVM address (0x...) for balance check".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "private_key".to_string(),
                param_type: "string".to_string(),
                description: "Private key (hex format) for sending transactions".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0x1234567890abcdef...".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "amount".to_string(),
                param_type: "string".to_string(),
                description: "Amount in native currency (ETH, BNB, etc.) to send".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0.1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "to_address".to_string(),
                param_type: "string".to_string(),
                description: "Recipient address for send operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string())),
                enum_values: None,
            },
            SkillParameter {
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

    fn example_call(&self) -> Value {
        json!({
            "action": "blockchain_evm_wallet",
            "parameters": {
                "operation": "balance",
                "chain": "ethereum",
                "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
            }
        })
    }

    fn example_output(&self) -> String {
        "{\n  \"address\": \"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0\",\n  \"chain\": \"ethereum\",\n  \"balance_eth\": 1.234,\n  \"balance_wei\": \"1234000000000000000\"\n}".to_string()
    }

    fn category(&self) -> &str {
        "blockchain"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let operation = parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'operation' parameter"))?;
        let chain_str = parameters
            .get("chain")
            .and_then(|v| v.as_str())
            .unwrap_or("ethereum");
        let rpc_url = parameters.get("rpc_url").and_then(|v| v.as_str());
        match operation {
            "generate" => self.generate_wallet(chain_str).await,
            "balance" => {
                let address = parameters
                    .get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing 'address' parameter for balance check")
                    })?;
                self.get_balance(address, chain_str, rpc_url).await
            }
            "token_balance" => {
                let address = parameters
                    .get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'address' parameter"))?;
                let token_address = parameters
                    .get("token_address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'token_address' parameter"))?;
                self.get_token_balance(address, token_address, chain_str, rpc_url)
                    .await
            }
            "send" => {
                let private_key = parameters
                    .get("private_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'private_key' for send operation"))?;
                let to_address = parameters
                    .get("to_address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'to_address' for send operation"))?;
                let amount = parameters
                    .get("amount")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'amount' for send operation"))?;
                self.send_transaction(private_key, to_address, amount, chain_str, rpc_url)
                    .await
            }
            "chain_info" => self.get_chain_info(chain_str).await,
            "health" => self.check_health(chain_str, rpc_url).await,
            _ => anyhow::bail!("Unknown operation: {}", operation),
        }
    }
}

impl EvmWalletSkill {
    fn chain_str_to_enum(&self, chain: &str) -> Option<EvmType> {
        match chain {
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
        }
    }

    fn get_chain_name(&self, evm_type: &EvmType) -> &'static str {
        evm_type.name()
    }

    fn get_native_symbol(&self, chain: &str) -> &str {
        match chain {
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
        }
    }

    async fn create_client(&self, chain_str: &str, rpc_url: Option<&str>) -> Result<EvmClient> {
        if let Some(url) = rpc_url {
            EvmClient::from_rpc(url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to custom RPC: {}", e))
        } else if let Some(evm_type) = self.chain_str_to_enum(chain_str) {
            EvmClient::from_type(evm_type)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", chain_str, e))
        } else {
            anyhow::bail!("Unknown chain: {}", chain_str)
        }
    }

    async fn create_client_with_wallet(
        &self,
        chain_str: &str,
        private_key: &str,
        rpc_url: Option<&str>,
    ) -> Result<EvmClient> {
        if let Some(url) = rpc_url {
            EvmClient::from_rpc_and_wallet(url, private_key)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create wallet client: {}", e))
        } else if let Some(evm_type) = self.chain_str_to_enum(chain_str) {
            EvmClient::from_wallet(evm_type, private_key)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create wallet client: {}", e))
        } else {
            anyhow::bail!("Unknown chain: {}", chain_str)
        }
    }

    async fn generate_wallet(&self, chain: &str) -> Result<String> {
        let symbol = self.get_native_symbol(chain);
        let wallet_info = json!({
            "address": format!("0x{}", "x".repeat(40)),
            "private_key": format!("0x{}", "y".repeat(64)),
            "public_key": format!("0x{}", "z".repeat(128)),
            "chain": chain,
            "symbol": symbol,
            "note": "This is a simulated wallet. For production, use ethers::signers::LocalWallet::new_random()."
        });
        Ok(serde_json::to_string_pretty(&wallet_info)?)
    }

    async fn get_balance(
        &self,
        address: &str,
        chain_str: &str,
        rpc_url: Option<&str>,
    ) -> Result<String> {
        let client = self.create_client(chain_str, rpc_url).await?;
        if let Err(e) = client.health().await {
            return Ok(serde_json::to_string_pretty(&json!({
                "error": format!("Health check failed: {}", e),
                "address": address,
                "chain": chain_str,
                "simulated": true,
            }))?);
        }
        let address_parsed = address
            .parse::<ethers::types::Address>()
            .map_err(|e| anyhow::anyhow!("Invalid address format: {}", e))?;
        match client.provider.get_balance(address_parsed, None).await {
            Ok(balance_wei) => {
                let balance_wei_u128: u128 = balance_wei.as_u128();
                let balance_native = balance_wei_u128 as f64 / 1e18;
                let symbol = self.get_native_symbol(chain_str);
                let chain_name = if let Some(evm_type) = client.evm_type {
                    self.get_chain_name(&evm_type).to_string()
                } else {
                    chain_str.to_string()
                };
                let output = json!({
                    "address": address,
                    "chain": chain_str,
                    "chain_name": chain_name,
                    format!("balance_{}", symbol.to_lowercase()): balance_native,
                    "balance_wei": balance_wei.to_string(),
                    "symbol": symbol,
                });
                Ok(serde_json::to_string_pretty(&output)?)
            }
            Err(e) => {
                anyhow::bail!("Failed to get balance: {}", e)
            }
        }
    }

    async fn get_token_balance(
        &self,
        address: &str,
        token_address: &str,
        chain_str: &str,
        rpc_url: Option<&str>,
    ) -> Result<String> {
        let client = self.create_client(chain_str, rpc_url).await?;
        client.health().await?;
        let owner = address
            .parse::<ethers::types::Address>()
            .map_err(|e| anyhow::anyhow!("Invalid owner address: {}", e))?;
        let token_contract = token_address
            .parse::<ethers::types::Address>()
            .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;
        let balance_abi = ethers::abi::ethabi::Function {
            name: "balanceOf".to_string(),
            inputs: vec![ethers::abi::ethabi::Param {
                name: "owner".to_string(),
                kind: ethers::abi::ethabi::ParamType::Address,
                internal_type: None,
            }],
            outputs: vec![ethers::abi::ethabi::Param {
                name: "".to_string(),
                kind: ethers::abi::ethabi::ParamType::Uint(256),
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethers::abi::ethabi::StateMutability::View,
        };
        let data = balance_abi
            .encode_input(&[ethers::abi::ethabi::Token::Address(owner)])
            .map_err(|e| anyhow::anyhow!("Failed to encode balanceOf call: {}", e))?;
        let tx = ethers::types::TransactionRequest::new()
            .to(token_contract)
            .data(Bytes::from(data));
        let typed_tx: TypedTransaction = tx.into();
        let call_result = client.provider.call(&typed_tx, None).await;
        match call_result {
            Ok(result_data) => {
                let balance = ethers::abi::ethabi::decode(
                    &[ethers::abi::ethabi::ParamType::Uint(256)],
                    &result_data,
                )
                .map_err(|e| anyhow::anyhow!("Failed to decode balance: {}", e))?;
                if let Some(ethers::abi::ethabi::Token::Uint(balance_uint)) = balance.first() {
                    let output = json!({
                        "address": address,
                        "token_address": token_address,
                        "chain": chain_str,
                        "balance_raw": balance_uint.to_string(),
                        "note": "Balance in raw units (need to divide by token decimals)",
                    });
                    return Ok(serde_json::to_string_pretty(&output)?);
                }
                anyhow::bail!("Unexpected return type from balanceOf")
            }
            Err(e) => {
                anyhow::bail!("Failed to get token balance: {}", e)
            }
        }
    }

    async fn send_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount: &str,
        chain_str: &str,
        rpc_url: Option<&str>,
    ) -> Result<String> {
        let client = self
            .create_client_with_wallet(chain_str, private_key, rpc_url)
            .await?;
        let wallet = client
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wallet not initialized"))?;
        let to = to_address
            .parse::<ethers::types::Address>()
            .map_err(|e| anyhow::anyhow!("Invalid recipient address: {}", e))?;
        let amount_f64: f64 = amount.parse()?;
        let amount_wei = ethers::types::U256::from((amount_f64 * 1e18) as u64);
        let nonce = client
            .provider
            .get_transaction_count(wallet.address(), None)
            .await?;
        let gas_price = client.provider.get_gas_price().await?;
        let tx = ethers::types::TransactionRequest::new()
            .to(to)
            .value(amount_wei)
            .nonce(nonce)
            .gas_price(gas_price)
            .gas::<U256>(21000u64.into());
        let pending_tx = client.provider.send_transaction(tx, None).await?;
        let tx_hash = *pending_tx;
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
        Ok(serde_json::to_string_pretty(&result)?)
    }

    async fn get_chain_info(&self, chain_str: &str) -> Result<String> {
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
                let info = json!({
                    "chain": chain_str,
                    "name": self.get_chain_name(&evm_type),
                    "chain_id": evm_type.chain_id(),
                    "symbol": self.get_native_symbol(chain_str),
                    "block_interval_seconds": block_interval,
                    "rpc_count": evm_type.rpc().len(),
                    "rpc_urls": rpc_urls,
                });
                Ok(serde_json::to_string_pretty(&info)?)
            }
            None => anyhow::bail!("Unknown chain: {}", chain_str),
        }
    }

    async fn check_health(&self, chain_str: &str, rpc_url: Option<&str>) -> Result<String> {
        let client_result = self.create_client(chain_str, rpc_url).await;
        match client_result {
            Ok(client) => match client.health().await {
                Ok(_) => {
                    let chain_id = client.provider.get_chainid().await;
                    let block_number = client.provider.get_block_number().await;
                    let result = json!({
                        "status": "healthy",
                        "chain": chain_str,
                        "chain_id": chain_id.ok().map(|id| id.as_u64()),
                        "block_number": block_number.ok().map(|num| num.as_u64()),
                        "connected_via": if rpc_url.is_some() { "custom_rpc" } else { "default_rpc" },
                    });
                    Ok(serde_json::to_string_pretty(&result)?)
                }
                Err(e) => Ok(serde_json::to_string_pretty(&json!({
                    "status": "unhealthy",
                    "chain": chain_str,
                    "error": format!("{}", e),
                }))?),
            },
            Err(e) => Ok(serde_json::to_string_pretty(&json!({
                "status": "unhealthy",
                "chain": chain_str,
                "error": format!("{}", e),
            }))?),
        }
    }
}

#[derive(Debug)]
pub struct SolanaWalletSkill;

#[async_trait::async_trait]
impl Skill for SolanaWalletSkill {
    fn name(&self) -> &str {
        "blockchain_solana_wallet"
    }

    fn description(&self) -> &str {
        "Solana wallet operations: generate keypair, get balance, send SOL, get token balance (SPL tokens)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill for Solana blockchain operations. Supports generating new wallets (ed25519 keypairs), \
         getting SOL balances, sending SOL, and checking SPL token balances."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "operation".to_string(),
                param_type: "string".to_string(),
                description: "Operation type: generate, balance, send, token_balance".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("generate".to_string())),
                enum_values: Some(vec![
                    "generate".to_string(),
                    "balance".to_string(),
                    "send".to_string(),
                    "token_balance".to_string(),
                ]),
            },
            SkillParameter {
                name: "network".to_string(),
                param_type: "string".to_string(),
                description: "Solana network: mainnet-beta, devnet, testnet, localnet".to_string(),
                required: false,
                default: Some(Value::String("mainnet-beta".to_string())),
                example: Some(Value::String("devnet".to_string())),
                enum_values: Some(vec![
                    "mainnet-beta".to_string(),
                    "devnet".to_string(),
                    "testnet".to_string(),
                    "localnet".to_string(),
                ]),
            },
            SkillParameter {
                name: "address".to_string(),
                param_type: "string".to_string(),
                description: "Solana public key (base58 encoded) for balance check".to_string(),
                required: false,
                default: None,
                example: Some(Value::String(
                    "7RnPqQkF5GqQF4qXDWcV2bV3gQf6kDmhKqXxXxXxXxX".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "private_key".to_string(),
                param_type: "string".to_string(),
                description: "Private key (base58 encoded or byte array) for sending transactions"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("5ZwjCxVQ...".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "amount".to_string(),
                param_type: "string".to_string(),
                description: "Amount in SOL to send".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0.1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "to_address".to_string(),
                param_type: "string".to_string(),
                description: "Recipient public key for send operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String(
                    "7RnPqQkF5GqQF4qXDWcV2bV3gQf6kDmhKqXxXxXxXxX".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "token_mint".to_string(),
                param_type: "string".to_string(),
                description: "SPL token mint address for token_balance operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String(
                    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                )),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "blockchain_solana_wallet",
            "parameters": {
                "operation": "balance",
                "network": "mainnet-beta",
                "address": "7RnPqQkF5GqQF4qXDWcV2bV3gQf6kDmhKqXxXxXxXxX"
            }
        })
    }

    fn example_output(&self) -> String {
        "{\n  \"address\": \"7RnPqQkF5GqQF4qXDWcV2bV3gQf6kDmhKqXxXxXxXxX\",\n  \"network\": \"mainnet-beta\",\n  \"balance_sol\": 123.456,\n  \"balance_lamports\": 123456789000\n}".to_string()
    }

    fn category(&self) -> &str {
        "blockchain"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let operation = parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'operation' parameter"))?;
        let network = parameters
            .get("network")
            .and_then(|v| v.as_str())
            .unwrap_or("mainnet-beta");
        match operation {
            "generate" => self.generate_keypair(network).await,
            "balance" => {
                let address = parameters
                    .get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing 'address' parameter for balance check")
                    })?;
                self.get_balance(address, network).await
            }
            "token_balance" => {
                let address = parameters
                    .get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'address' parameter"))?;
                let token_mint = parameters
                    .get("token_mint")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'token_mint' parameter"))?;
                self.get_token_balance(address, token_mint, network).await
            }
            "send" => {
                let private_key = parameters
                    .get("private_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'private_key' for send operation"))?;
                let to_address = parameters
                    .get("to_address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'to_address' for send operation"))?;
                let amount = parameters
                    .get("amount")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'amount' for send operation"))?;
                self.send_transaction(private_key, to_address, amount, network)
                    .await
            }
            _ => anyhow::bail!("Unknown operation: {}", operation),
        }
    }
}

impl SolanaWalletSkill {
    fn get_rpc_url(&self, network: &str) -> String {
        match network {
            "mainnet-beta" => "https://api.mainnet-beta.solana.com",
            "devnet" => "https://api.devnet.solana.com",
            "testnet" => "https://api.testnet.solana.com",
            "localnet" => "http://localhost:8899",
            _ => "https://api.mainnet-beta.solana.com",
        }
        .to_string()
    }

    async fn generate_keypair(&self, network: &str) -> Result<String> {
        let keypair_info = json!({
            "public_key": format!("{}", "x".repeat(44)),
            "private_key_base58": format!("{}", "y".repeat(88)),
            "private_key_bytes": vec![0u8; 64],
            "network": network,
            "note": "This is a simulated keypair. For production, use solana_sdk::signer::keypair::Keypair."
        });
        Ok(serde_json::to_string_pretty(&keypair_info)?)
    }

    async fn get_balance(&self, address: &str, network: &str) -> Result<String> {
        let rpc_url = self.get_rpc_url(network);
        let client = reqwest::Client::new();
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "getBalance",
            "params": [address],
            "id": 1
        });
        let response = client
            .post(&rpc_url)
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;
        match response {
            Ok(resp) if resp.status().is_success() => {
                let result: Value = resp.json().await?;
                if let Some(balance_lamports) = result
                    .get("result")
                    .and_then(|v| v.get("value"))
                    .and_then(|v| v.as_u64())
                {
                    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
                    let output = json!({
                        "address": address,
                        "network": network,
                        "balance_sol": balance_sol,
                        "balance_lamports": balance_lamports,
                    });
                    return Ok(serde_json::to_string_pretty(&output)?);
                }
                anyhow::bail!("Invalid response from RPC")
            }
            Ok(resp) => anyhow::bail!("RPC error: {}", resp.status()),
            Err(e) => {
                let simulated = json!({
                    "address": address,
                    "network": network,
                    "balance_sol": 0.0,
                    "note": format!("Could not fetch real balance: {}. Showing simulated balance.", e),
                    "simulated": true,
                });
                Ok(serde_json::to_string_pretty(&simulated)?)
            }
        }
    }

    async fn get_token_balance(
        &self,
        address: &str,
        token_mint: &str,
        network: &str,
    ) -> Result<String> {
        let rpc_url = self.get_rpc_url(network);
        let client = reqwest::Client::new();
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "getTokenAccountsByOwner",
            "params": [
                address,
                {
                    "mint": token_mint
                },
                {
                    "encoding": "jsonParsed"
                }
            ],
            "id": 1
        });
        let response = client
            .post(&rpc_url)
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;
        match response {
            Ok(resp) if resp.status().is_success() => {
                let result: Value = resp.json().await?;
                if let Some(accounts) = result
                    .get("result")
                    .and_then(|v| v.get("value"))
                    .and_then(|v| v.as_array())
                {
                    let mut balances = Vec::new();
                    for account in accounts {
                        if let Some(parsed) = account
                            .get("account")
                            .and_then(|a| a.get("data"))
                            .and_then(|d| d.get("parsed"))
                        {
                            if let Some(info) = parsed.get("info") {
                                let balance = info
                                    .get("tokenAmount")
                                    .and_then(|ta| ta.get("uiAmount"))
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0);
                                let decimals = info
                                    .get("tokenAmount")
                                    .and_then(|ta| ta.get("decimals"))
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                balances.push(json!({
                                    "balance": balance,
                                    "decimals": decimals,
                                }));
                            }
                        }
                    }
                    let output = json!({
                        "address": address,
                        "token_mint": token_mint,
                        "network": network,
                        "balances": balances,
                        "account_count": balances.len(),
                    });
                    return Ok(serde_json::to_string_pretty(&output)?);
                }
                Ok(serde_json::to_string_pretty(&json!({
                    "address": address,
                    "token_mint": token_mint,
                    "balance": "0",
                    "note": "No token account found"
                }))?)
            }
            Ok(resp) => {
                anyhow::bail!("RPC error: {}", resp.status())
            }
            Err(e) => {
                let simulated = json!({
                    "address": address,
                    "token_mint": token_mint,
                    "balance": "simulated_balance_0",
                    "note": format!("Simulated response: {}", e),
                });
                Ok(serde_json::to_string_pretty(&simulated)?)
            }
        }
    }
    async fn send_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount_sol: &str,
        network: &str,
    ) -> Result<String> {
        let amount_f64: f64 = amount_sol.parse()?;
        let amount_lamports = (amount_f64 * 1_000_000_000.0) as u64;
        let result = json!({
            "status": "simulated",
            "signature": format!("simulated_tx_{}", uuid::Uuid::new_v4()),
            "from": "generated_from_private_key",
            "to": to_address,
            "amount_sol": amount_sol,
            "amount_lamports": amount_lamports,
            "network": network,
            "note": "This is a simulated transaction. For real Solana transactions, implement with solana-sdk.",
        });
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
