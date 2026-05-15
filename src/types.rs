#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub response: String,
    pub matched: bool,
    pub skill_name: Option<String>,
}
