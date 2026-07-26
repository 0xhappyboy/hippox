//! GitHub API driver module
//!
//! This module provides drivers for GitHub API operations including
//! repository information, issue management, searching, and user information.
use crate::DriverCallback;
use crate::DriverContext;
use crate::RequestConfig;
use crate::execute;
use crate::types::{Driver, DriverParameter};
use crate::{DriverCategory, DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
// ========== Helper functions ==========
/// Retrieves a string parameter from the parameters map
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
///
/// # Returns
/// * `DriverResult<String>` - Parameter value on success
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    return params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| DriverError::missing_parameter(name));
}
/// Retrieves a u64 parameter from the parameters map with a default value
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
/// * `default` - Default value if parameter is not present
///
/// # Returns
/// * `u64` - Parameter value or default
fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    return params.get(name).and_then(|v| v.as_u64()).unwrap_or(default);
}
/// Retrieves an array parameter from the parameters map
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
///
/// # Returns
/// * `Vec<Value>` - Array value or empty vector
fn get_param_array(params: &HashMap<String, Value>, name: &str) -> Vec<Value> {
    return params.get(name).and_then(|v| v.as_array()).cloned().unwrap_or_default();
}
/// Retrieves a boolean parameter from the parameters map with a default value
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
/// * `default` - Default value if parameter is not present
///
/// # Returns
/// * `bool` - Parameter value or default
fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    return params.get(name).and_then(|v| v.as_bool()).unwrap_or(default);
}
// ========== GitHub API Helper ==========
/// GitHub API client wrapper
struct GitHubApi;
impl GitHubApi {
    /// Builds the full URL for a GitHub API endpoint
    ///
    /// # Arguments
    /// * `api_url` - Base API URL
    /// * `endpoint` - API endpoint path
    ///
    /// # Returns
    /// * `String` - Full URL
    fn build_url(api_url: &str, endpoint: &str) -> String {
        return format!("{}/{}", api_url.trim_end_matches('/'), endpoint);
    }
    /// Builds HTTP headers for GitHub API requests
    ///
    /// # Arguments
    /// * `token` - GitHub personal access token
    ///
    /// # Returns
    /// * `HashMap<String, String>` - HTTP headers
    fn build_headers(token: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "application/vnd.github.v3+json".to_string());
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        headers.insert("User-Agent".to_string(), "Hippox-Engine".to_string());
        return headers;
    }
    /// Performs a GET request to the GitHub API
    ///
    /// # Arguments
    /// * `endpoint` - API endpoint path
    /// * `token` - GitHub personal access token
    /// * `api_url` - Base API URL
    /// * `timeout` - Request timeout in seconds
    ///
    /// # Returns
    /// * `DriverResult<String>` - Response body on success
    async fn get(endpoint: &str, token: &str, api_url: &str, timeout: u64) -> DriverResult<String> {
        debug!("GitHub GET request to: {}", endpoint);
        let req_config = RequestConfig {
            url: Self::build_url(api_url, endpoint),
            method: "GET".to_string(),
            headers: Some(Self::build_headers(token)),
            body: None,
            timeout_secs: Some(timeout),
        };
        let response = execute(&req_config).await.map_err(|e| DriverError::execution(format!("GitHub API request failed: {}", e)))?;
        if response.is_success {
            info!("GitHub GET request successful: {}", endpoint);
            return Ok(response.body);
        } else {
            return Err(DriverError::execution(format!("GitHub API error: {}", response.body)));
        }
    }
    /// Performs a POST request to the GitHub API
    ///
    /// # Arguments
    /// * `endpoint` - API endpoint path
    /// * `body` - Request body
    /// * `token` - GitHub personal access token
    /// * `api_url` - Base API URL
    /// * `timeout` - Request timeout in seconds
    ///
    /// # Returns
    /// * `DriverResult<String>` - Response body on success
    async fn post(endpoint: &str, body: &str, token: &str, api_url: &str, timeout: u64) -> DriverResult<String> {
        debug!("GitHub POST request to: {}", endpoint);
        let req_config = RequestConfig {
            url: Self::build_url(api_url, endpoint),
            method: "POST".to_string(),
            headers: Some(Self::build_headers(token)),
            body: Some(body.to_string()),
            timeout_secs: Some(timeout),
        };
        let response = execute(&req_config).await.map_err(|e| DriverError::execution(format!("GitHub API request failed: {}", e)))?;
        if response.is_success {
            info!("GitHub POST request successful: {}", endpoint);
            return Ok(response.body);
        } else {
            return Err(DriverError::execution(format!("GitHub API error: {}", response.body)));
        }
    }
    /// Performs a PUT request to the GitHub API
    ///
    /// # Arguments
    /// * `endpoint` - API endpoint path
    /// * `body` - Request body (optional)
    /// * `token` - GitHub personal access token
    /// * `api_url` - Base API URL
    /// * `timeout` - Request timeout in seconds
    ///
    /// # Returns
    /// * `DriverResult<String>` - Response body on success
    async fn put(endpoint: &str, body: Option<&str>, token: &str, api_url: &str, timeout: u64) -> DriverResult<String> {
        debug!("GitHub PUT request to: {}", endpoint);
        let req_config = RequestConfig {
            url: Self::build_url(api_url, endpoint),
            method: "PUT".to_string(),
            headers: Some(Self::build_headers(token)),
            body: body.map(|s| s.to_string()),
            timeout_secs: Some(timeout),
        };
        let response = execute(&req_config).await.map_err(|e| DriverError::execution(format!("GitHub API request failed: {}", e)))?;
        if response.is_success {
            info!("GitHub PUT request successful: {}", endpoint);
            return Ok(response.body);
        } else {
            return Err(DriverError::execution(format!("GitHub API error: {}", response.body)));
        }
    }
    /// Performs a DELETE request to the GitHub API
    ///
    /// # Arguments
    /// * `endpoint` - API endpoint path
    /// * `token` - GitHub personal access token
    /// * `api_url` - Base API URL
    /// * `timeout` - Request timeout in seconds
    ///
    /// # Returns
    /// * `DriverResult<String>` - Response body on success
    async fn delete(endpoint: &str, token: &str, api_url: &str, timeout: u64) -> DriverResult<String> {
        debug!("GitHub DELETE request to: {}", endpoint);
        let req_config = RequestConfig {
            url: Self::build_url(api_url, endpoint),
            method: "DELETE".to_string(),
            headers: Some(Self::build_headers(token)),
            body: None,
            timeout_secs: Some(timeout),
        };
        let response = execute(&req_config).await.map_err(|e| DriverError::execution(format!("GitHub API request failed: {}", e)))?;
        if response.is_success {
            info!("GitHub DELETE request successful: {}", endpoint);
            return Ok(response.body);
        } else {
            return Err(DriverError::execution(format!("GitHub API error: {}", response.body)));
        }
    }
}
// ========== Get repository information ==========
/// Driver for getting GitHub repository information
#[derive(Debug)]
pub struct GithubGetRepo;
#[async_trait::async_trait]
impl Driver for GithubGetRepo {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "github_get_repo";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get information about a GitHub repository";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to get repository details like stars, forks, description";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner (username or organization)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL (default: https://api.github.com)".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "github_get_repo",
            "parameters": {
                "token": "ghp_xxxxxxxx",
                "owner": "rust-lang",
                "repo": "rust"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"{"name": "rust", "full_name": "rust-lang/rust", "description": "Empowering everyone...", "stargazers_count": 85000, "forks_count": 11000}"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing github_get_repo driver");
        // Extract required parameters
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let api_url = parameters.get("api_url").and_then(|v| v.as_str()).unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);
        let endpoint = format!("repos/{}/{}", owner, repo);
        debug!("Fetching repository info: {}/{}", owner, repo);
        let result = GitHubApi::get(&endpoint, &token, api_url, timeout).await?;
        info!("Successfully fetched repository info: {}/{}", owner, repo);
        return Ok(result);
    }
}
// ========== Create an issue ==========
/// Driver for creating a GitHub issue
#[derive(Debug)]
pub struct GithubCreateIssue;
#[async_trait::async_trait]
impl Driver for GithubCreateIssue {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "github_create_issue";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Create an issue in a GitHub repository";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to report a bug or request a feature";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Issue title".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Bug: compilation error".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "body".to_string(),
                param_type: "string".to_string(),
                description: "Issue body/description".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("When compiling with nightly...".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "labels".to_string(),
                param_type: "array".to_string(),
                description: "Labels to apply".to_string(),
                required: false,
                default: Some(Value::Array(vec![])),
                example: Some(json!(["bug", "help-wanted"])),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "github_create_issue",
            "parameters": {
                "token": "ghp_xxxxxxxx",
                "owner": "rust-lang",
                "repo": "rust",
                "title": "Bug: compilation error",
                "body": "When compiling with nightly...",
                "labels": ["bug"]
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"{"number": 12345, "html_url": "https://github.com/rust-lang/rust/issues/12345"}"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing github_create_issue driver");
        // Extract required parameters
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let title = get_param_string(parameters, "title")?;
        let body = parameters.get("body").and_then(|v| v.as_str());
        let labels = get_param_array(parameters, "labels");
        let api_url = parameters.get("api_url").and_then(|v| v.as_str()).unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);
        // Build request body
        let mut body_json = json!({ "title": title });
        if let Some(b) = body {
            body_json["body"] = json!(b);
        }
        if !labels.is_empty() {
            let label_strings: Vec<String> = labels.iter().filter_map(|l| l.as_str()).map(|s| s.to_string()).collect();
            body_json["labels"] = json!(label_strings);
        }
        let endpoint = format!("repos/{}/{}/issues", owner, repo);
        debug!("Creating issue in {}/{}: {}", owner, repo, title);
        let result = GitHubApi::post(&endpoint, &body_json.to_string(), &token, api_url, timeout).await?;
        info!("Successfully created issue in {}/{}", owner, repo);
        return Ok(result);
    }
}
// ========== List issues ==========
/// Driver for listing GitHub issues
#[derive(Debug)]
pub struct GithubListIssues;
#[async_trait::async_trait]
impl Driver for GithubListIssues {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "github_list_issues";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "List issues from a GitHub repository";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to see existing issues";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "state".to_string(),
                param_type: "string".to_string(),
                description: "Issue state (open, closed, all)".to_string(),
                required: false,
                default: Some(Value::String("open".to_string())),
                example: Some(Value::String("open".to_string())),
                enum_values: Some(vec!["open".to_string(), "closed".to_string(), "all".to_string()]),
            },
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of issues to return".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "github_list_issues",
            "parameters": {
                "token": "ghp_xxxxxxxx",
                "owner": "rust-lang",
                "repo": "rust",
                "state": "open",
                "limit": 10
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"[{"number": 12345, "title": "Bug report", "state": "open"}]"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing github_list_issues driver");
        // Extract required parameters
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let state = parameters.get("state").and_then(|v| v.as_str()).unwrap_or("open");
        let limit = get_param_u64(parameters, "limit", 30);
        let api_url = parameters.get("api_url").and_then(|v| v.as_str()).unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);
        let endpoint = format!("repos/{}/{}/issues?state={}&per_page={}", owner, repo, state, limit);
        debug!("Listing issues from {}/{} with state: {}", owner, repo, state);
        let result = GitHubApi::get(&endpoint, &token, api_url, timeout).await?;
        info!("Successfully listed issues from {}/{}", owner, repo);
        return Ok(result);
    }
}
// ========== Star a repository ==========
/// Driver for starring a GitHub repository
#[derive(Debug)]
pub struct GithubStarRepo;
#[async_trait::async_trait]
impl Driver for GithubStarRepo {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "github_star_repo";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Star a GitHub repository";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to star/favorite a repository";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "github_star_repo",
            "parameters": {
                "token": "ghp_xxxxxxxx",
                "owner": "rust-lang",
                "repo": "rust"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully starred rust-lang/rust".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing github_star_repo driver");
        // Extract required parameters
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let api_url = parameters.get("api_url").and_then(|v| v.as_str()).unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);
        let endpoint = format!("user/starred/{}/{}", owner, repo);
        debug!("Starring repository: {}/{}", owner, repo);
        GitHubApi::put(&endpoint, None, &token, api_url, timeout).await?;
        info!("Successfully starred {}/{}", owner, repo);
        return Ok(format!("Successfully starred {}/{}", owner, repo));
    }
}
// ========== Search repositories ==========
/// Driver for searching GitHub repositories
#[derive(Debug)]
pub struct GithubSearchRepos;
#[async_trait::async_trait]
impl Driver for GithubSearchRepos {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "github_search_repos";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Search GitHub repositories by query";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to find repositories";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "Search query (e.g., 'rust language:rust')".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust language:rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of results".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "github_search_repos",
            "parameters": {
                "token": "ghp_xxxxxxxx",
                "query": "rust language:rust",
                "limit": 5
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"{"total_count": 12345, "items": [{"full_name": "rust-lang/rust", "description": "..."}]}"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing github_search_repos driver");
        // Extract required parameters
        let token = get_param_string(parameters, "token")?;
        let query = get_param_string(parameters, "query")?;
        let limit = get_param_u64(parameters, "limit", 10);
        let api_url = parameters.get("api_url").and_then(|v| v.as_str()).unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);
        let encoded_query = urlencoding::encode(&query);
        let endpoint = format!("search/repositories?q={}&per_page={}", encoded_query, limit);
        debug!("Searching repositories with query: {}", query);
        let result = GitHubApi::get(&endpoint, &token, api_url, timeout).await?;
        info!("Successfully searched repositories with query: {}", query);
        return Ok(result);
    }
}
// ========== Get user information ==========
/// Driver for getting GitHub user information
#[derive(Debug)]
pub struct GithubGetUser;
#[async_trait::async_trait]
impl Driver for GithubGetUser {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "github_get_user";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get GitHub user information";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to get profile info of a GitHub user";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "GitHub username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("octocat".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "github_get_user",
            "parameters": {
                "token": "ghp_xxxxxxxx",
                "username": "octocat"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"{"login": "octocat", "name": "The Octocat", "public_repos": 8}"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing github_get_user driver");
        // Extract required parameters
        let token = get_param_string(parameters, "token")?;
        let username = get_param_string(parameters, "username")?;
        let api_url = parameters.get("api_url").and_then(|v| v.as_str()).unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);
        let endpoint = format!("users/{}", username);
        debug!("Fetching user info: {}", username);
        let result = GitHubApi::get(&endpoint, &token, api_url, timeout).await?;
        info!("Successfully fetched user info: {}", username);
        return Ok(result);
    }
}
// ========== List pull requests ==========
/// Driver for listing GitHub pull requests
#[derive(Debug)]
pub struct GithubListPRs;
#[async_trait::async_trait]
impl Driver for GithubListPRs {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "github_list_prs";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "List pull requests from a GitHub repository";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to see open pull requests";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Devops;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "state".to_string(),
                param_type: "string".to_string(),
                description: "PR state (open, closed, all)".to_string(),
                required: false,
                default: Some(Value::String("open".to_string())),
                example: Some(Value::String("open".to_string())),
                enum_values: Some(vec!["open".to_string(), "closed".to_string(), "all".to_string()]),
            },
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of PRs to return".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "github_list_prs",
            "parameters": {
                "token": "ghp_xxxxxxxx",
                "owner": "rust-lang",
                "repo": "rust",
                "state": "open",
                "limit": 10
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"[{"number": 123, "title": "Add feature", "user": {"login": "contributor"}}]"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing github_list_prs driver");
        // Extract required parameters
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let state = parameters.get("state").and_then(|v| v.as_str()).unwrap_or("open");
        let limit = get_param_u64(parameters, "limit", 30);
        let api_url = parameters.get("api_url").and_then(|v| v.as_str()).unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);
        let endpoint = format!("repos/{}/{}/pulls?state={}&per_page={}", owner, repo, state, limit);
        debug!("Listing PRs from {}/{} with state: {}", owner, repo, state);
        let result = GitHubApi::get(&endpoint, &token, api_url, timeout).await?;
        info!("Successfully listed PRs from {}/{}", owner, repo);
        return Ok(result);
    }
}
