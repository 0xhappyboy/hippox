//! Math calculator skill
//!
//! This driver provides mathematical expression evaluation with support for
//! arithmetic operations, trigonometric functions, logarithms, constants, and more.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverError;
use crate::DriverResult;
use crate::{
    DriverCategory, format_number,
    types::{Driver, DriverParameter},
    validate_number,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Mathematical expression evaluator with support for:
/// - Basic arithmetic: +, -, *, /, %
/// - Functions: sin, cos, tan, asin, acos, atan, log, ln, sqrt, abs, floor, ceil, round, factorial
/// - Constants: pi, e
/// - Parentheses and operator precedence
/// - Scientific notation: 1e-5, 2.5e3
#[derive(Debug)]
pub struct CalculatorDriver;
#[async_trait::async_trait]
impl Driver for CalculatorDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "math_calculator";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Evaluate mathematical expressions with support for arithmetic, trigonometric functions, logarithms, constants, and more";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user asks to calculate, compute, or solve a math expression. \
         Supports: + - * / % ^, sin, cos, tan, asin, acos, atan, log (base 10), ln (natural log), \
         sqrt, abs, floor, ceil, round, factorial, constants pi and e, scientific notation (e.g., 1e-5).";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "expression".to_string(),
                param_type: "string".to_string(),
                description: "Math expression to evaluate. Examples: '2 + 3 * 4', 'sin(pi/2)', 'log(100)', '5!', 'abs(-5)', 'floor(3.7)', '2e-3'"
                    .to_string(),
                required: true,
                default: None,
                example: Some(Value::String("sin(pi/2) + log(100)".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "precision".to_string(),
                param_type: "integer".to_string(),
                description: "Number of decimal places in the result (default: 6)".to_string(),
                required: false,
                default: Some(Value::Number(6.into())),
                example: Some(Value::Number(4.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "degrees".to_string(),
                param_type: "boolean".to_string(),
                description: "Use degrees for trigonometric functions (default: false, uses radians)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "math_calculator",
            "parameters": {
                "expression": "2 + 3 * 4",
                "precision": 2
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "14.00".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Math;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing math_calculator driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting math calculation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let expression = parameters.get("expression").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'expression' parameter");
            return DriverError::missing_parameter("expression");
        })?;
        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Expression: {}", expression)));
            cb.on_progress(task_id.clone(), driver_index, Some(25), None);
        }
        let use_degrees = parameters.get("degrees").and_then(|v| v.as_bool()).unwrap_or(false);
        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Using degrees: {}", use_degrees)));
            cb.on_progress(task_id.clone(), driver_index, Some(45), None);
        }
        debug!("Evaluating expression: {} (degrees: {})", expression, use_degrees);
        let result = evaluate_expression(expression, use_degrees).map_err(|e| {
            warn!("Failed to evaluate expression: {}", e);
            return DriverError::execution(format!("Failed to evaluate expression: {}", e));
        })?;
        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Raw result: {}", result)));
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        let precision = parameters.get("precision").and_then(|v| v.as_u64()).unwrap_or(6);
        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Formatting with precision: {}", precision)));
            cb.on_progress(task_id.clone(), driver_index, Some(90), None);
        }
        let res = format_number(result, precision as usize);
        info!("Calculation result: {}", res);
        if let Some(cb) = cb {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Final result: {}", res)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("math_calculator".to_string()), Some(res.clone()));
        }
        return Ok(res);
    }
    /// Validates parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        if parameters.get("expression").and_then(|v| v.as_str()).is_none() {
            return Err(DriverError::missing_parameter("expression"));
        }
        return Ok(());
    }
}
/// Main expression evaluator entry point
fn evaluate_expression(expr: &str, use_degrees: bool) -> Result<f64, String> {
    let expr: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    let expr = replace_constants(&expr);
    let expr = expand_scientific_notation(&expr);
    let expr = expand_factorial(&expr)?;
    let expr = evaluate_functions(&expr, use_degrees)?;
    if expr.contains('(') {
        return evaluate_with_parentheses(&expr, use_degrees);
    }
    return evaluate_basic(&expr);
}
/// Replace constants (pi, e) with their numeric values
fn replace_constants(expr: &str) -> String {
    let mut result = expr.to_string();
    result = result.replace("π", "pi");
    result = result.replace("PI", "pi");
    result = result.replace("Pi", "pi");
    result = result.replace("pi", &std::f64::consts::PI.to_string());
    result = result.replace("e", &std::f64::consts::E.to_string());
    return result;
}
/// Expand scientific notation like 1e-5 to 0.00001
fn expand_scientific_notation(expr: &str) -> String {
    let re = regex::Regex::new(r"(\d+(?:\.\d+)?)e([+-]?\d+)").unwrap();
    return re
        .replace_all(expr, |caps: &regex::Captures| {
            let mantissa: f64 = caps[1].parse().unwrap_or(0.0);
            let exponent: i32 = caps[2].parse().unwrap_or(0);
            (mantissa * 10f64.powi(exponent)).to_string()
        })
        .to_string();
}
/// Expand factorial notation (e.g., 5! -> 120)
fn expand_factorial(expr: &str) -> Result<String, String> {
    let re = regex::Regex::new(r"(\d+(?:\.\d+)?)!").unwrap();
    let result = re
        .replace_all(expr, |caps: &regex::Captures| {
            let num: f64 = caps[1].parse().unwrap_or(0.0);
            if num.fract() != 0.0 {
                return "NaN".to_string();
            }
            let n = num as u64;
            if n > 20 {
                return "Infinity".to_string();
            }
            (1..=n).product::<u64>().to_string()
        })
        .to_string();
    let re2 = regex::Regex::new(r"\(([^()]+)\)!").unwrap();
    let result = re2
        .replace_all(&result, |caps: &regex::Captures| {
            let inner = &caps[1];
            format!("factorial({})", inner)
        })
        .to_string();
    return Ok(result);
}
/// Evaluate function calls like sin(30), log(100), sqrt(16)
fn evaluate_functions(expr: &str, use_degrees: bool) -> Result<String, String> {
    let functions = vec!["sin", "cos", "tan", "asin", "acos", "atan", "log", "ln", "sqrt", "abs", "floor", "ceil", "round"];
    let mut result = expr.to_string();
    for func in functions {
        let pattern = format!(r"{}\(([^()]+(?:\([^()]*\)[^()]*)*)\)", func);
        let re = regex::Regex::new(&pattern).unwrap();
        while let Some(caps) = re.captures(&result) {
            let full_match = caps[0].to_string();
            let inner_expr = caps[1].to_string();
            let inner_value = evaluate_basic(&inner_expr)?;
            let computed = match func {
                "sin" => {
                    let rad = if use_degrees { inner_value.to_radians() } else { inner_value };
                    rad.sin()
                }
                "cos" => {
                    let rad = if use_degrees { inner_value.to_radians() } else { inner_value };
                    rad.cos()
                }
                "tan" => {
                    let rad = if use_degrees { inner_value.to_radians() } else { inner_value };
                    rad.tan()
                }
                "asin" => {
                    let val = inner_value.asin();
                    if use_degrees { val.to_degrees() } else { val }
                }
                "acos" => {
                    let val = inner_value.acos();
                    if use_degrees { val.to_degrees() } else { val }
                }
                "atan" => {
                    let val = inner_value.atan();
                    if use_degrees { val.to_degrees() } else { val }
                }
                "log" => inner_value.log10(),
                "ln" => inner_value.ln(),
                "sqrt" => {
                    if inner_value < 0.0 {
                        return Err(format!("Cannot take square root of negative number: {}", inner_value));
                    }
                    inner_value.sqrt()
                }
                "abs" => inner_value.abs(),
                "floor" => inner_value.floor(),
                "ceil" => inner_value.ceil(),
                "round" => inner_value.round(),
                _ => inner_value,
            };
            result = result.replace(&full_match, &computed.to_string());
        }
    }
    return Ok(result);
}
/// Evaluate expressions with parentheses
fn evaluate_with_parentheses(expr: &str, use_degrees: bool) -> Result<f64, String> {
    let mut expr = expr.to_string();
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
        let inner_result = evaluate_expression(inner, use_degrees)?;
        let new_expr = format!("{}{}{}", &expr[..s], inner_result, &expr[e + 1..]);
        return evaluate_expression(&new_expr, use_degrees);
    }
    return evaluate_basic(&expr);
}
/// Evaluate basic arithmetic expression without parentheses
fn evaluate_basic(expr: &str) -> Result<f64, String> {
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
    for i in 0..chars.len() {
        if chars[i] == '+' {
            signs.push(true);
        } else if chars[i] == '-' {
            signs.push(false);
        }
    }
    let mut term_values = Vec::new();
    for term in terms {
        let value = evaluate_term(term)?;
        term_values.push(value);
    }
    let mut result = term_values[0];
    for i in 1..term_values.len() {
        let is_add = if i - 1 < signs.len() { signs[i - 1] } else { true };
        if is_add {
            result += term_values[i];
        } else {
            result -= term_values[i];
        }
    }
    return Ok(result);
}
/// Evaluate a term (contains *, /, % operators)
fn evaluate_term(term: &str) -> Result<f64, String> {
    let chars: Vec<char> = term.chars().collect();
    let mut factors = Vec::new();
    let mut start = 0;
    for i in 0..chars.len() {
        if chars[i] == '*' || chars[i] == '/' || chars[i] == '%' || chars[i] == '^' {
            if start < i {
                factors.push(&term[start..i]);
            }
            start = i + 1;
        }
    }
    if start < term.len() {
        factors.push(&term[start..]);
    }
    let mut result = match validate_number(factors[0]) {
        Ok(v) => v,
        Err(e) => return Err(format!("Invalid number: {}", e)),
    };
    let mut op_index = 0;
    for i in 0..chars.len() {
        if chars[i] == '*' {
            let next = match validate_number(factors[op_index + 1]) {
                Ok(v) => v,
                Err(e) => return Err(format!("Invalid number: {}", e)),
            };
            result *= next;
            op_index += 1;
        } else if chars[i] == '/' {
            let next = match validate_number(factors[op_index + 1]) {
                Ok(v) => v,
                Err(e) => return Err(format!("Invalid number: {}", e)),
            };
            if next == 0.0 {
                return Err("Division by zero".to_string());
            }
            result /= next;
            op_index += 1;
        } else if chars[i] == '%' {
            let next = match validate_number(factors[op_index + 1]) {
                Ok(v) => v,
                Err(e) => return Err(format!("Invalid number: {}", e)),
            };
            result %= next;
            op_index += 1;
        } else if chars[i] == '^' {
            let next = match validate_number(factors[op_index + 1]) {
                Ok(v) => v,
                Err(e) => return Err(format!("Invalid number: {}", e)),
            };
            result = result.powf(next);
            op_index += 1;
        }
    }
    return Ok(result);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_arithmetic() {
        let result = evaluate_expression("2+3*4", false).unwrap();
        assert_eq!(result, 14.0);
        let result = evaluate_expression("(2+3)*4", false).unwrap();
        assert_eq!(result, 20.0);
        let result = evaluate_expression("10/2", false).unwrap();
        assert_eq!(result, 5.0);
        let result = evaluate_expression("10%3", false).unwrap();
        assert_eq!(result, 1.0);
    }
    #[test]
    fn test_power() {
        let result = evaluate_expression("2^3", false).unwrap();
        assert_eq!(result, 8.0);
        let result = evaluate_expression("4^0.5", false).unwrap();
        assert_eq!(result, 2.0);
    }
    #[test]
    fn test_constants() {
        let result = evaluate_expression("pi", false).unwrap();
        assert_eq!(result, std::f64::consts::PI);
        let result = evaluate_expression("e", false).unwrap();
        assert_eq!(result, std::f64::consts::E);
    }
    #[test]
    fn test_scientific_notation() {
        let result = evaluate_expression("1e-3", false).unwrap();
        assert_eq!(result, 0.001);
        let result = evaluate_expression("2.5e2", false).unwrap();
        assert_eq!(result, 250.0);
    }
    #[test]
    fn test_factorial() {
        let result = evaluate_expression("5!", false).unwrap();
        assert_eq!(result, 120.0);
        let result = evaluate_expression("0!", false).unwrap();
        assert_eq!(result, 1.0);
    }
    #[test]
    fn test_trig_functions_radians() {
        let result = evaluate_expression("sin(pi/2)", false).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
        let result = evaluate_expression("cos(0)", false).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_trig_functions_degrees() {
        let result = evaluate_expression("sin(90)", true).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
        let result = evaluate_expression("cos(0)", true).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_logarithms() {
        let result = evaluate_expression("log(100)", false).unwrap();
        assert!((result - 2.0).abs() < 1e-10);
        let result = evaluate_expression("ln(e)", false).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_sqrt() {
        let result = evaluate_expression("sqrt(16)", false).unwrap();
        assert_eq!(result, 4.0);
        let result = evaluate_expression("sqrt(2)", false).unwrap();
        assert!((result - 1.41421356237).abs() < 1e-6);
    }
    #[test]
    fn test_abs() {
        let result = evaluate_expression("abs(-5)", false).unwrap();
        assert_eq!(result, 5.0);
        let result = evaluate_expression("abs(3)", false).unwrap();
        assert_eq!(result, 3.0);
    }
    #[test]
    fn test_floor_ceil_round() {
        let result = evaluate_expression("floor(3.7)", false).unwrap();
        assert_eq!(result, 3.0);
        let result = evaluate_expression("ceil(3.2)", false).unwrap();
        assert_eq!(result, 4.0);
        let result = evaluate_expression("round(3.5)", false).unwrap();
        assert_eq!(result, 4.0);
    }
    #[test]
    fn test_complex_expression() {
        let result = evaluate_expression("sin(pi/2) + log(100) + 2^3", false).unwrap();
        assert!((result - (1.0 + 2.0 + 8.0)).abs() < 1e-10);
        let result = evaluate_expression("(5+3)! / 2", false).unwrap();
        assert_eq!(result, 20160.0);
    }
}
