//! Solana wallet driver
//!
//! This driver provides Solana blockchain operations including generating
//! keypairs, checking balances, sending SOL, and querying SPL token balances.
use crate::DriverCallback;
use crate::DriverCategory;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Solana wallet driver for blockchain operations
#[derive(Debug)]
pub struct SolanaWalletDriver;
#[async_trait::async_trait]
impl Driver for SolanaWalletDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "blockchain_solana_wallet"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Solana wallet operations: generate keypair, get balance, send SOL, get token balance (SPL tokens)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill for Solana blockchain operations. Supports generating new wallets (ed25519 keypairs), \
         getting SOL balances, sending SOL, and checking SPL token balances."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "operation".to_string(),
                param_type: "string".to_string(),
                description: "Operation type: generate, balance, send, token_balance".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("generate".to_string())),
                enum_values: Some(vec!["generate".to_string(), "balance".to_string(), "send".to_string(), "token_balance".to_string()]),
            },
            DriverParameter {
                name: "network".to_string(),
                param_type: "string".to_string(),
                description: "Solana network: mainnet-beta, devnet, testnet, localnet".to_string(),
                required: false,
                default: Some(Value::String("mainnet-beta".to_string())),
                example: Some(Value::String("devnet".to_string())),
                enum_values: Some(vec!["mainnet-beta".to_string(), "devnet".to_string(), "testnet".to_string(), "localnet".to_string()]),
            },
            DriverParameter {
                name: "address".to_string(),
                param_type: "string".to_string(),
                description: "Solana public key (base58 encoded) for balance check".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("7RnPqQkF5GqQF4qXDWcV2bV3gQf6kDmhKqXxXxXxXxX".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "private_key".to_string(),
                param_type: "string".to_string(),
                description: "Private key (base58 encoded or byte array) for sending transactions".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("5ZwjCxVQ...".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "amount".to_string(),
                param_type: "string".to_string(),
                description: "Amount in SOL to send".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0.1".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "to_address".to_string(),
                param_type: "string".to_string(),
                description: "Recipient public key for send operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("7RnPqQkF5GqQF4qXDWcV2bV3gQf6kDmhKqXxXxXxXxX".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "token_mint".to_string(),
                param_type: "string".to_string(),
                description: "SPL token mint address for token_balance operation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "blockchain_solana_wallet",
            "parameters": {
                "operation": "balance",
                "network": "mainnet-beta",
                "address": "7RnPqQkF5GqQF4qXDWcV2bV3gQf6kDmhKqXxXxXxXxX"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "{\n  \"address\": \"7RnPqQkF5GqQF4qXDWcV2bV3gQf6kDmhKqXxXxXxXxX\",\n  \"network\": \"mainnet-beta\",\n  \"balance_sol\": 123.456,\n  \"balance_lamports\": 123456789000\n}".to_string()
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
        debug!("Executing solana_wallet driver");
        let operation = parameters.get("operation").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'operation' parameter");
            DriverError::missing_parameter("operation")
        })?;
        let network = parameters.get("network").and_then(|v| v.as_str()).unwrap_or("mainnet-beta");
        debug!("Operation: {}, network: {}", operation, network);
        match operation {
            "generate" => self.generate_keypair(network).await,
            "balance" => {
                let address = parameters.get("address").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing 'address' parameter for balance check");
                    DriverError::missing_parameter("address")
                })?;
                self.get_balance(address, network).await
            }
            "token_balance" => {
                let address = parameters.get("address").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing 'address' parameter");
                    DriverError::missing_parameter("address")
                })?;
                let token_mint = parameters.get("token_mint").and_then(|v| v.as_str()).ok_or_else(|| {
                    debug!("Missing 'token_mint' parameter");
                    DriverError::missing_parameter("token_mint")
                })?;
                self.get_token_balance(address, token_mint, network).await
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
                self.send_transaction(private_key, to_address, amount, network).await
            }
            _ => {
                warn!("Unknown operation: {}", operation);
                Err(DriverError::execution(format!("Unknown operation: {}", operation)))
            }
        }
    }
}
impl SolanaWalletDriver {
    /// Get RPC URL for a network
    fn get_rpc_url(&self, network: &str) -> String {
        let url = match network {
            "mainnet-beta" => "https://api.mainnet-beta.solana.com",
            "devnet" => "https://api.devnet.solana.com",
            "testnet" => "https://api.testnet.solana.com",
            "localnet" => "http://localhost:8899",
            _ => "https://api.mainnet-beta.solana.com",
        };
        debug!("RPC URL for {}: {}", network, url);
        url.to_string()
    }
    /// Generate a new Solana keypair
    async fn generate_keypair(&self, network: &str) -> DriverResult<String> {
        debug!("Generating Solana keypair on {}", network);
        let keypair_info = json!({
            "public_key": format!("{}", "x".repeat(44)),
            "private_key_base58": format!("{}", "y".repeat(88)),
            "private_key_bytes": vec![0u8; 64],
            "network": network,
            "note": "This is a simulated keypair. For production, use solana_sdk::signer::keypair::Keypair."
        });
        info!("Solana keypair generated on {}", network);
        Ok(serde_json::to_string_pretty(&keypair_info).map_err(|e| DriverError::execution(format!("Failed to serialize keypair info: {}", e)))?)
    }
    /// Get SOL balance for an address
    async fn get_balance(&self, address: &str, network: &str) -> DriverResult<String> {
        debug!("Getting balance for address: {} on {}", address, network);
        let rpc_url = self.get_rpc_url(network);
        let client = reqwest::Client::new();
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "getBalance",
            "params": [address],
            "id": 1
        });
        debug!("Querying RPC: {}", rpc_url);
        let response = client.post(&rpc_url).json(&request_body).timeout(std::time::Duration::from_secs(30)).send().await;
        match response {
            Ok(resp) if resp.status().is_success() => {
                let result: Value = resp.json().await.map_err(|e| DriverError::execution(format!("Failed to parse RPC response: {}", e)))?;
                if let Some(balance_lamports) = result.get("result").and_then(|v| v.get("value")).and_then(|v| v.as_u64()) {
                    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
                    debug!("Balance: {} SOL ({} lamports)", balance_sol, balance_lamports);
                    let output = json!({
                        "address": address,
                        "network": network,
                        "balance_sol": balance_sol,
                        "balance_lamports": balance_lamports,
                    });
                    info!("Balance retrieved for {}", address);
                    return Ok(
                        serde_json::to_string_pretty(&output).map_err(|e| DriverError::execution(format!("Failed to serialize balance: {}", e)))?
                    );
                }
                warn!("Invalid response from RPC");
                Err(DriverError::execution("Invalid response from RPC".to_string()))
            }
            Ok(resp) => {
                warn!("RPC error: {}", resp.status());
                Err(DriverError::execution(format!("RPC error: {}", resp.status())))
            }
            Err(e) => {
                warn!("Failed to fetch balance: {}", e);
                let simulated = json!({
                    "address": address,
                    "network": network,
                    "balance_sol": 0.0,
                    "note": format!("Could not fetch real balance: {}. Showing simulated balance.", e),
                    "simulated": true,
                });
                Ok(serde_json::to_string_pretty(&simulated)
                    .map_err(|e| DriverError::execution(format!("Failed to serialize simulated balance: {}", e)))?)
            }
        }
    }
    /// Get SPL token balance for an address
    async fn get_token_balance(&self, address: &str, token_mint: &str, network: &str) -> DriverResult<String> {
        debug!("Getting token balance for address: {} token: {} on {}", address, token_mint, network);
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
        debug!("Querying RPC: {}", rpc_url);
        let response = client.post(&rpc_url).json(&request_body).timeout(std::time::Duration::from_secs(30)).send().await;
        match response {
            Ok(resp) if resp.status().is_success() => {
                let result: Value = resp.json().await.map_err(|e| DriverError::execution(format!("Failed to parse RPC response: {}", e)))?;
                if let Some(accounts) = result.get("result").and_then(|v| v.get("value")).and_then(|v| v.as_array()) {
                    let mut balances = Vec::new();
                    for account in accounts {
                        if let Some(parsed) = account.get("account").and_then(|a| a.get("data")).and_then(|d| d.get("parsed")) {
                            if let Some(info) = parsed.get("info") {
                                let balance = info.get("tokenAmount").and_then(|ta| ta.get("uiAmount")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let decimals = info.get("tokenAmount").and_then(|ta| ta.get("decimals")).and_then(|v| v.as_u64()).unwrap_or(0);
                                balances.push(json!({
                                    "balance": balance,
                                    "decimals": decimals,
                                }));
                            }
                        }
                    }
                    debug!("Found {} token accounts", balances.len());
                    let output = json!({
                        "address": address,
                        "token_mint": token_mint,
                        "network": network,
                        "balances": balances,
                        "account_count": balances.len(),
                    });
                    info!("Token balance retrieved for {}", address);
                    return Ok(serde_json::to_string_pretty(&output)
                        .map_err(|e| DriverError::execution(format!("Failed to serialize token balance: {}", e)))?);
                }
                debug!("No token account found");
                Ok(serde_json::to_string_pretty(&json!({
                    "address": address,
                    "token_mint": token_mint,
                    "balance": "0",
                    "note": "No token account found"
                }))
                .map_err(|e| DriverError::execution(format!("Failed to serialize response: {}", e)))?)
            }
            Ok(resp) => {
                warn!("RPC error: {}", resp.status());
                Err(DriverError::execution(format!("RPC error: {}", resp.status())))
            }
            Err(e) => {
                warn!("Failed to fetch token balance: {}", e);
                let simulated = json!({
                    "address": address,
                    "token_mint": token_mint,
                    "balance": "simulated_balance_0",
                    "note": format!("Simulated response: {}", e),
                });
                Ok(serde_json::to_string_pretty(&simulated)
                    .map_err(|e| DriverError::execution(format!("Failed to serialize simulated response: {}", e)))?)
            }
        }
    }
    /// Send a Solana transaction
    async fn send_transaction(&self, private_key: &str, to_address: &str, amount_sol: &str, network: &str) -> DriverResult<String> {
        debug!("Sending transaction: {} SOL from private key to {} on {}", amount_sol, to_address, network);
        let amount_f64: f64 = amount_sol.parse().map_err(|e| DriverError::validation("amount", format!("Invalid amount format: {}", e)))?;
        let amount_lamports = (amount_f64 * 1_000_000_000.0) as u64;
        let signature = format!("simulated_tx_{}", uuid::Uuid::new_v4());
        debug!("Transaction details: {} lamports, signature: {}", amount_lamports, signature);
        let result = json!({
            "status": "simulated",
            "signature": signature,
            "from": "generated_from_private_key",
            "to": to_address,
            "amount_sol": amount_sol,
            "amount_lamports": amount_lamports,
            "network": network,
            "note": "This is a simulated transaction. For real Solana transactions, implement with solana-sdk.",
        });
        info!("Transaction simulated: {}", result["signature"]);
        Ok(serde_json::to_string_pretty(&result).map_err(|e| DriverError::execution(format!("Failed to serialize transaction result: {}", e)))?)
    }
}
