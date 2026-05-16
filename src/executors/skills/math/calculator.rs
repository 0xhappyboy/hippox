use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{
    skills::common,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct CalculatorSkill;

#[async_trait::async_trait]
impl Skill for CalculatorSkill {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate mathematical expressions"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user asks to calculate, compute, or solve a math expression"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "expression".to_string(),
                param_type: "string".to_string(),
                description: "Math expression to evaluate, e.g., '2 + 3 * 4' or '(10 - 5) / 2'"
                    .to_string(),
                required: true,
                default: None,
                example: Some(Value::String("2 + 3 * 4".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "precision".to_string(),
                param_type: "integer".to_string(),
                description: "Number of decimal places in the result".to_string(),
                required: false,
                default: Some(Value::Number(2.into())),
                example: Some(Value::Number(2.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "calculator",
            "parameters": {
                "expression": "2 + 3 * 4",
                "precision": 2
            }
        })
    }

    fn example_output(&self) -> String {
        "14.00".to_string()
    }

    fn category(&self) -> &str {
        "math"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let expression = parameters
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'expression' parameter"))?;
        let result = evaluate_expression(expression)?;
        let precision = parameters
            .get("precision")
            .and_then(|v| v.as_u64())
            .unwrap_or(2);
        Ok(common::Math::format_number(result, precision as usize))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: expression"))?;
        Ok(())
    }
}

// Rest of the expression evaluator functions remain the same...
fn evaluate_expression(expr: &str) -> Result<f64> {
    let expr: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    if expr.contains('(') {
        return evaluate_with_parentheses(&expr);
    }
    evaluate_basic(&expr)
}

fn evaluate_basic(expr: &str) -> Result<f64> {
    let expr = expr.to_string();
    let chars: Vec<char> = expr.chars().collect();
    let mut terms = Vec::new();
    let mut start = 0;
    for i in 0..chars.len() {
        if chars[i] == '+' || chars[i] == '-' {
            if start < i {
                terms.push(&expr[start..i]);
            }
            start = i + 1;
        }
    }
    if start < expr.len() {
        terms.push(&expr[start..]);
    }
    let mut signs = Vec::new();
    start = 0;
    for i in 0..chars.len() {
        if chars[i] == '+' {
            signs.push(true);
            start = i + 1;
        } else if chars[i] == '-' {
            signs.push(false);
            start = i + 1;
        }
    }
    let mut term_values = Vec::new();
    for term in terms {
        let value = evaluate_term(term)?;
        term_values.push(value);
    }
    let mut result = term_values[0];
    for i in 1..term_values.len() {
        let is_add = if i - 1 < signs.len() {
            signs[i - 1]
        } else {
            true
        };
        if is_add {
            result += term_values[i];
        } else {
            result -= term_values[i];
        }
    }
    Ok(result)
}

fn evaluate_term(term: &str) -> Result<f64> {
    let chars: Vec<char> = term.chars().collect();
    let mut factors = Vec::new();
    let mut start = 0;
    for i in 0..chars.len() {
        if chars[i] == '*' || chars[i] == '/' || chars[i] == '%' {
            if start < i {
                factors.push(&term[start..i]);
            }
            start = i + 1;
        }
    }
    if start < term.len() {
        factors.push(&term[start..]);
    }
    let mut result = common::Math::validate_number(factors[0])?;
    let mut op_index = 0;
    for i in 0..chars.len() {
        if chars[i] == '*' {
            let next = common::Math::validate_number(factors[op_index + 1])?;
            result *= next;
            op_index += 1;
        } else if chars[i] == '/' {
            let next = common::Math::validate_number(factors[op_index + 1])?;
            if next == 0.0 {
                anyhow::bail!("Division by zero");
            }
            result /= next;
            op_index += 1;
        } else if chars[i] == '%' {
            let next = common::Math::validate_number(factors[op_index + 1])?;
            result %= next;
            op_index += 1;
        }
    }
    Ok(result)
}

fn evaluate_with_parentheses(expr: &str) -> Result<f64> {
    let mut start = None;
    let mut end = None;
    for (i, c) in expr.chars().enumerate() {
        if c == '(' {
            start = Some(i);
        } else if c == ')' {
            if let Some(s) = start {
                end = Some(i);
                break;
            }
        }
    }
    if let (Some(s), Some(e)) = (start, end) {
        let inner = &expr[s + 1..e];
        let inner_result = evaluate_expression(inner)?;
        let new_expr = format!("{}{}{}", &expr[..s], inner_result, &expr[e + 1..]);
        return evaluate_expression(&new_expr);
    }
    evaluate_basic(expr)
}
