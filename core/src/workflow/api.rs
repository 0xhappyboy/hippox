use crate::WorkflowMode;

/// Get workflow mode display name in English
pub fn workflow_mode_display_name_en(mode: WorkflowMode) -> &'static str {
    match mode {
        WorkflowMode::ReAct => "ReAct",
        WorkflowMode::Batch => "Batch",
        WorkflowMode::Chain => "Chain",
        WorkflowMode::PlanAndExecute => "PlanAndExecute",
    }
}

/// Get workflow mode display name in Chinese
pub fn workflow_mode_display_name_zh(mode: WorkflowMode) -> &'static str {
    match mode {
        WorkflowMode::ReAct => "反应式",
        WorkflowMode::Batch => "批量式",
        WorkflowMode::Chain => "链式",
        WorkflowMode::PlanAndExecute => "计划执行式",
    }
}

/// Get workflow mode display name by language
pub fn workflow_mode_display_name_by_lang(mode: WorkflowMode, lang: &str) -> &'static str {
    match lang {
        "zh" | "zh-CN" | "zh-TW" => workflow_mode_display_name_zh(mode),
        _ => workflow_mode_display_name_en(mode),
    }
}

/// Get workflow mode names in English (for dropdown/list)
pub fn get_workflow_mode_names_en() -> Vec<&'static str> {
    vec!["ReAct", "Batch", "Chain", "PlanAndExecute"]
}

/// Get workflow mode names in Chinese (for dropdown/list)
pub fn get_workflow_mode_names_zh() -> Vec<&'static str> {
    vec!["反应式", "批量式", "链式", "计划执行式"]
}

/// Convert string to WorkflowMode
pub fn string_to_workflow_mode(s: &str) -> Option<WorkflowMode> {
    match s {
        "ReAct" | "react" => Some(WorkflowMode::ReAct),
        "Batch" | "batch" => Some(WorkflowMode::Batch),
        "Chain" | "chain" => Some(WorkflowMode::Chain),
        "PlanAndExecute" | "plan_and_execute" | "plan" => Some(WorkflowMode::PlanAndExecute),
        _ => None,
    }
}

/// Get WorkflowMode as string (lowercase)
pub fn workflow_mode_to_string(mode: WorkflowMode) -> String {
    match mode {
        WorkflowMode::ReAct => "react".to_string(),
        WorkflowMode::Batch => "batch".to_string(),
        WorkflowMode::Chain => "chain".to_string(),
        WorkflowMode::PlanAndExecute => "plan_and_execute".to_string(),
    }
}

/// Get WorkflowMode display name (default to English)
pub fn workflow_mode_display_name(mode: WorkflowMode) -> &'static str {
    workflow_mode_display_name_en(mode)
}

/// Get WorkflowMode description in Chinese
pub fn workflow_mode_description_zh(mode: WorkflowMode) -> &'static str {
    match mode {
        WorkflowMode::ReAct => {
            "思考 → 行动 → 观察循环模式，每次执行后由 LLM 决策下一步，最适合开放式任务、动态决策和错误恢复"
        }
        WorkflowMode::Batch => {
            "批量并行执行多个独立的驱动，驱动之间无依赖关系，最适合批量处理和独立操作"
        }
        WorkflowMode::Chain => "链式串行执行，驱动间可传递变量，最适合线性流水线和数据转换链",
        WorkflowMode::PlanAndExecute => {
            "先规划后执行模式，支持条件判断、变量引用和错误处理，最适合复杂工作流和确定性任务"
        }
    }
}

/// Get WorkflowMode description in English
pub fn workflow_mode_description_en(mode: WorkflowMode) -> &'static str {
    match mode {
        WorkflowMode::ReAct => {
            "Think → Act → Observe loop mode. Each driver execution is followed by LLM decision for next step. Best for open-ended tasks, dynamic decision making, and error recovery."
        }
        WorkflowMode::Batch => {
            "Execute multiple independent drivers in parallel. Drivers have no dependencies on each other. Best for batch processing and independent operations."
        }
        WorkflowMode::Chain => {
            "Sequential execution with variable passing between drivers. Best for linear pipelines and data transformation chains."
        }
        WorkflowMode::PlanAndExecute => {
            "One-time planning with full workflow support. Supports conditionals, variable references, and error handling. Best for complex workflows and deterministic tasks."
        }
    }
}
