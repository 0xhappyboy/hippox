//! Error handler prompt templates for ReAct mode

/// Build error feedback prompt for failed skill execution
///
/// Provides structured error information to LLM so it can make informed decisions
/// about how to proceed with the workflow.
///
/// # Arguments
/// * `skill_name` - Name of the skill that failed
/// * `error_msg` - The error message from skill execution
/// * `attempt` - Current attempt number (1-indexed)
/// * `max_retries` - Maximum allowed retries
/// * `parameters` - The parameters that were used
///
/// # Returns
/// A formatted prompt that guides LLM to make a decision
pub fn build_error_feedback_prompt(
    skill_name: &str,
    error_msg: &str,
    attempt: usize,
    max_retries: usize,
    parameters: &serde_json::Value,
) -> String {
    let remaining_attempts = max_retries.saturating_sub(attempt);

    format!(
        r#"❌ Skill Execution Failed

## Failed Skill
- **Name**: `{}`
- **Parameters**: `{}`
- **Attempt**: {}/{}
- **Remaining Retries**: {}

## Error Details
    {}
    
## What You Can Do Now
You MUST choose ONE of the following actions:

1. **Retry with different parameters**
   - Adjust the parameters and try again
   - Example: change file path, modify search query, etc.

2. **Use a different skill**
   - Choose an alternative skill that can accomplish the same goal
   - Consider skills with similar functionality

3. **Skip this step and continue**
   - Move on to the next step in the workflow
   - The result of this step will be treated as a warning

4. **Abort and return error message**
   - Stop the workflow and return an error to the user
   - Only if the failure is critical and cannot be recovered

## Decision Format
Respond with a valid skill call JSON:

### Option 1: Retry (same skill, different params)
{{"action": "{}", "parameters": {{"param1": "new_value"}}}}

### Option 2: Different skill
{{"action": "alternative_skill", "parameters": {{}}}}

### Option 3: Skip
{{"action": "done", "message": "Skipped {} due to error: {}"}}

### Option 4: Abort
{{"action": "done", "message": "Unable to complete: {}"}}

## Your Decision
"#,
        skill_name,
        serde_json::to_string_pretty(parameters).unwrap_or_default(),
        attempt,
        max_retries,
        remaining_attempts,
        error_msg,
        skill_name,
        skill_name,
        error_msg,
        error_msg
    )
}

/// Build timeout feedback prompt for skill timeout
///
/// # Arguments
/// * `skill_name` - Name of the skill that timed out
/// * `timeout_secs` - The timeout duration in seconds
/// * `attempt` - Current attempt number (1-indexed)
/// * `max_retries` - Maximum allowed retries
///
/// # Returns
/// A formatted prompt that guides LLM to make a decision
pub fn build_timeout_feedback_prompt(
    skill_name: &str,
    timeout_secs: u64,
    attempt: usize,
    max_retries: usize,
) -> String {
    let remaining_attempts = max_retries.saturating_sub(attempt);

    format!(
        r#"⏰ Skill Timeout

## Timed Out Skill
- **Name**: `{}`
- **Timeout**: {} seconds
- **Attempt**: {}/{}
- **Remaining Retries**: {}

## What Happened
The skill took too long to complete and was automatically stopped.

## What You Can Do Now
You MUST choose ONE of the following actions:

1. **Retry with timeout adjustment**
   - The skill might need more time or different parameters
   - Consider simplifying the request

2. **Use a different skill**
   - Choose an alternative skill that might be faster
   - Look for skills with similar functionality

3. **Skip this step and continue**
   - Move on to the next step
   - The timeout will be logged as a warning

4. **Abort and return error message**
   - Stop the workflow and return an error to the user

## Decision Format
Respond with a valid skill call JSON:

### Option 1: Retry (same skill)
{{"action": "{}", "parameters": {{"input": "simplified request"}}}}

### Option 2: Different skill
{{"action": "alternative_skill", "parameters": {{}}}}

### Option 3: Skip
{{"action": "done", "message": "Skipped {} due to timeout"}}

### Option 4: Abort
{{"action": "done", "message": "Operation timed out: {}"}}

## Your Decision
"#,
        skill_name,
        timeout_secs,
        attempt,
        max_retries,
        remaining_attempts,
        skill_name,
        skill_name,
        skill_name
    )
}

/// Build max retries exceeded feedback prompt
///
/// # Arguments
/// * `skill_name` - Name of the skill that exceeded max retries
/// * `max_retries` - Maximum allowed retries
/// * `last_error` - The last error message
///
/// # Returns
/// A formatted prompt that forces a decision
pub fn build_max_retries_exceeded_prompt(
    skill_name: &str,
    max_retries: usize,
    last_error: &str,
) -> String {
    format!(
        r#"⚠️ Maximum Retries Exceeded

## Skill Information
- **Name**: `{}`
- **Max Retries**: {}
- **Last Error**: `{}`

## Status
This skill has been retried {} times and continues to fail.

## Mandatory Decision
You CANNOT retry this skill again. You MUST choose ONE of the following:

1. **Use a different skill**
   {{"action": "alternative_skill", "parameters": {{}}}}

2. **Skip this step and continue**
   {{"action": "done", "message": "Skipped {} after {} failed attempts"}}

3. **Abort and return error message**
   {{"action": "done", "message": "Failed to execute {}: {}"}}

## Your Decision
"#,
        skill_name,
        max_retries,
        last_error,
        max_retries,
        skill_name,
        max_retries,
        skill_name,
        last_error
    )
}

/// Build consecutive failures feedback prompt
///
/// # Arguments
/// * `failure_count` - Number of consecutive failures
/// * `max_failures` - Maximum allowed consecutive failures
///
/// # Returns
/// A formatted prompt warning about consecutive failures
pub fn build_consecutive_failures_prompt(failure_count: usize, max_failures: usize) -> String {
    format!(
        r#"⚠️ Consecutive Failures Detected

## Failure Count
- **Consecutive Failures**: {}/{}

## Warning
The system is experiencing repeated failures. This may indicate:
- The chosen approach is not working
- There might be a systemic issue
- Skills may be unavailable or misconfigured

## Action Required
You MUST make a significant change to break the failure cycle:

1. **Switch to a completely different approach**
   - Use a different category of skills
   - Consider if the task can be accomplished differently

2. **Return a clear error message to the user**
   - Explain what went wrong
   - Suggest alternative actions

## Decision Format
{{"action": "done", "message": "I'm having trouble with this task. Let me try a different approach."}}

## Your Decision
"#,
        failure_count, max_failures
    )
}
