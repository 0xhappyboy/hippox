use base64::Engine;
use langhub::types::LangHubResult;
use tracing::debug;

/// Download a URL to a local file.
pub async fn download_to_file(url: &str, dest: &std::path::Path) -> LangHubResult<()> {
    let resp = reqwest::get(url).await.map_err(|e| langhub::types::LangHubError::LLMError(format!("HTTP error: {}", e)))?;
    if !resp.status().is_success() {
        return Err(langhub::types::LangHubError::LLMError(format!("HTTP status {}", resp.status())));
    }
    let bytes = resp.bytes().await.map_err(|e| langhub::types::LangHubError::LLMError(format!("Read bytes error: {}", e)))?;
    std::fs::write(dest, &bytes).map_err(|e| langhub::types::LangHubError::IoError(e))?;
    debug!("Downloaded {} bytes to {:?}", bytes.len(), dest);
    Ok(())
}
/// Decode a base64 string and write it to a local file.
pub fn base64_decode_to_file(b64: &str, dest: &std::path::Path) -> LangHubResult<()> {
    let engine = base64::engine::general_purpose::STANDARD;
    let bytes = engine.decode(b64).map_err(|e| langhub::types::LangHubError::LLMError(format!("Base64 decode error: {}", e)))?;
    std::fs::write(dest, &bytes).map_err(|e| langhub::types::LangHubError::IoError(e))?;
    Ok(())
}
