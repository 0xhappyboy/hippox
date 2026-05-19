use hippox::{ConfigInitMethod, Hippox, ModelProvider, WorkflowMode};
use tempfile::tempdir;

/// Dynamic decision making, each step determined by previous result
async fn demo_react_mode(api_key: &str, temp_dir: &tempfile::TempDir) {
    let hippox = Hippox::with_workflow_mode(
        temp_dir.path().to_str().unwrap(),
        ModelProvider::DeepSeek,
        Some(api_key.to_string()),
        None,
        ConfigInitMethod::Env,
        WorkflowMode::ReAct,
    )
    .await
    .unwrap();
    let user_input = "Calculate (10 + 5) * 2 - 8 / 2 + 100, then tell me the result";
    println!("Input: {}", user_input);
    let result = hippox.handle_natural_language(user_input, None).await;
    println!("Result: {}\n", result);
}

/// Execute independent tasks in parallel
async fn demo_batch_mode(api_key: &str, temp_dir: &tempfile::TempDir) {
    let hippox = Hippox::with_workflow_mode(
        temp_dir.path().to_str().unwrap(),
        ModelProvider::DeepSeek,
        Some(api_key.to_string()),
        None,
        ConfigInitMethod::Env,
        WorkflowMode::Batch,
    )
    .await
    .unwrap();
    let user_input = "Calculate simultaneously: 15 * 3, 100 / 4, 89 + 11, 200 - 50";
    println!("Input: {}", user_input);
    let result = hippox.handle_natural_language(user_input, None).await;
    println!("Result: {}\n", result);
}

/// Chain calculation with results passed sequentially
async fn demo_chain_mode(api_key: &str, temp_dir: &tempfile::TempDir) {
    let hippox = Hippox::with_workflow_mode(
        temp_dir.path().to_str().unwrap(),
        ModelProvider::DeepSeek,
        Some(api_key.to_string()),
        None,
        ConfigInitMethod::Env,
        WorkflowMode::Chain,
    )
    .await
    .unwrap();
    let user_input =
        "Start from 5, multiply by 3, subtract 2, multiply by 4, divide by 2, then add 10";
    println!("Input: {}", user_input);
    let result = hippox.handle_natural_language(user_input, None).await;
    println!("Result: {}\n", result);
}

/// One-time planning for complex tasks
async fn demo_plan_and_execute_mode(api_key: &str, temp_dir: &tempfile::TempDir) {
    let hippox = Hippox::with_workflow_mode(
        temp_dir.path().to_str().unwrap(),
        ModelProvider::DeepSeek,
        Some(api_key.to_string()),
        None,
        ConfigInitMethod::Env,
        WorkflowMode::PlanAndExecute,
    )
    .await
    .unwrap();
    let user_input = "Calculate 2 to the power of 4, then subtract 10, then multiply by 3. If the result is greater than 50, subtract 20, otherwise add 30";
    println!("Input: {}", user_input);
    let result = hippox.handle_natural_language(user_input, None).await;
    println!("Result: {}\n", result);
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let temp_dir = tempdir().unwrap();
    let deep_seek_key = std::env::var("DEEP_SEEK_KEY").unwrap_or_default();
    if deep_seek_key.is_empty() {
        println!("Skipping: DEEP_SEEK_KEY not set");
        return;
    }
    demo_react_mode(&deep_seek_key, &temp_dir).await;
    demo_batch_mode(&deep_seek_key, &temp_dir).await;
    demo_chain_mode(&deep_seek_key, &temp_dir).await;
    demo_plan_and_execute_mode(&deep_seek_key, &temp_dir).await;
}
