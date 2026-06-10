//! Stage Two prompt template

/// Build format conversion prompt
///
/// This prompt instructs LLM to check user's format requirements
/// and convert the JSON output accordingly.
pub fn build_format_conversion_prompt(original_input: &str, json_output: &str) -> String {
    let prompt = format!(
        r##"
You are a format converter. Your ONLY job is to convert the JSON result below into the format requested by the user.

## User's Original Request (CHECK HERE FOR FORMAT REQUIREMENTS)
<<<USER_INPUT_START>>>
{}
<<<USER_INPUT_END>>>

## JSON Result to Convert (DO NOT CHANGE THE CONTENT)

{}

## Task Instructions
1. READ the user's original request above
2. If the user specifies a format or provides a structure template, convert the JSON result accordingly
3. If no format is specified, output the original JSON as-is

## Rules
- Do NOT change the data/content - ONLY change the presentation format
- Do NOT add any explanation or commentary
- Output ONLY the converted result (or original JSON)

## Your Output
"##,
        original_input, json_output
    );
    prompt
}
