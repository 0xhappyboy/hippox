#![allow(warnings)]
mod config;
mod core;
mod executors;
mod i18n;
mod pipeline;
mod prompts;
mod tasks;
mod workflow;

pub use crate::config::*;
pub use crate::core::*;
pub use crate::executors::skill_registry;
pub use crate::pipeline::*;
pub use crate::skill_scheduler::*;
pub use crate::tasks::*;
pub use crate::workflow::*;
pub use langhub::types::ModelProvider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        println!("8")
    }
}
