//! Response formatting prompt templates

/// Build format conversion prompt (without format specification)
pub fn build_format_conversion_prompt(original_input: &str, json_output: &str) -> String {
    format!(
        r##"
Convert the JSON below to the format requested by the user.

## User Request (check for format requirements)
{}
## JSON Result
{}
## Output ONLY the converted result. No explanations.
"##,
        original_input, json_output
    )
}
