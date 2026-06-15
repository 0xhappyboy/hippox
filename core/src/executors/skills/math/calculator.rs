use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{
    types::{Skill, SkillParameter},
    {format_number, validate_number},
};

/// Mathematical expression evaluator with support for:
/// - Basic arithmetic: +, -, *, /, %
/// - Functions: sin, cos, tan, asin, acos, atan, log, ln, sqrt, abs, floor, ceil, round, factorial
/// - Constants: pi, e
/// - Parentheses and operator precedence
/// - Scientific notation: 1e-5, 2.5e3
#[derive(Debug)]
pub struct CalculatorSkill;

#[async_trait::async_trait]
impl Skill for CalculatorSkill {
    fn name(&self) -> &str {
        "math_calculator"
    }

    fn description(&self) -> &str {
        "Evaluate mathematical expressions with support for arithmetic, trigonometric functions, logarithms, constants, and more"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user asks to calculate, compute, or solve a math expression. \
         Supports: + - * / % ^, sin, cos, tan, asin, acos, atan, log (base 10), ln (natural log), \
         sqrt, abs, floor, ceil, round, factorial, constants pi and e, scientific notation (e.g., 1e-5)."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "expression".to_string(),
                param_type: "string".to_string(),
                description: "Math expression to evaluate. Examples: '2 + 3 * 4', 'sin(pi/2)', 'log(100)', '5!', 'abs(-5)', 'floor(3.7)', '2e-3'".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("sin(pi/2) + log(100)".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "precision".to_string(),
                param_type: "integer".to_string(),
                description: "Number of decimal places in the result (default: 6)".to_string(),
                required: false,
                default: Some(Value::Number(6.into())),
                example: Some(Value::Number(4.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "degrees".to_string(),
                param_type: "boolean".to_string(),
                description: "Use degrees for trigonometric functions (default: false, uses radians)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "math_calculator",
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
        let use_degrees = parameters
            .get("degrees")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let result = evaluate_expression(expression, use_degrees)?;
        let precision = parameters
            .get("precision")
            .and_then(|v| v.as_u64())
            .unwrap_or(6);
        Ok(format_number(result, precision as usize))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: expression"))?;
        Ok(())
    }
}

/// Main expression evaluator entry point
fn evaluate_expression(expr: &str, use_degrees: bool) -> Result<f64> {
    let expr: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    let expr = replace_constants(&expr);
    let expr = expand_scientific_notation(&expr);
    let expr = expand_factorial(&expr)?;
    let expr = evaluate_functions(&expr, use_degrees)?;
    if expr.contains('(') {
        return evaluate_with_parentheses(&expr, use_degrees);
    }
    evaluate_basic(&expr)
}

/// Replace constants (pi, e) with their numeric values
fn replace_constants(expr: &str) -> String {
    let mut result = expr.to_string();
    result = result.replace("π", "pi");
    result = result.replace("PI", "pi");
    result = result.replace("Pi", "pi");
    result = result.replace("pi", &std::f64::consts::PI.to_string());
    result = result.replace("e", &std::f64::consts::E.to_string());
    result
}

/// Expand scientific notation like 1e-5 to 0.00001
fn expand_scientific_notation(expr: &str) -> String {
    let re = regex::Regex::new(r"(\d+(?:\.\d+)?)e([+-]?\d+)").unwrap();
    re.replace_all(expr, |caps: &regex::Captures| {
        let mantissa: f64 = caps[1].parse().unwrap_or(0.0);
        let exponent: i32 = caps[2].parse().unwrap_or(0);
        (mantissa * 10f64.powi(exponent)).to_string()
    })
    .to_string()
}

/// Expand factorial notation (e.g., 5! -> 120)
fn expand_factorial(expr: &str) -> Result<String> {
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
            // This needs to be evaluated recursively, but for now return placeholders
            format!("factorial({})", inner)
        })
        .to_string();
    Ok(result)
}

/// Evaluate function calls like sin(30), log(100), sqrt(16)
fn evaluate_functions(expr: &str, use_degrees: bool) -> Result<String> {
    let functions = vec![
        "sin", "cos", "tan", "asin", "acos", "atan", "log", "ln", "sqrt", "abs", "floor", "ceil",
        "round",
    ];
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
                    let rad = if use_degrees {
                        inner_value.to_radians()
                    } else {
                        inner_value
                    };
                    rad.sin()
                }
                "cos" => {
                    let rad = if use_degrees {
                        inner_value.to_radians()
                    } else {
                        inner_value
                    };
                    rad.cos()
                }
                "tan" => {
                    let rad = if use_degrees {
                        inner_value.to_radians()
                    } else {
                        inner_value
                    };
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
                        return Err(anyhow::anyhow!(
                            "Cannot take square root of negative number: {}",
                            inner_value
                        ));
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
    Ok(result)
}

/// Evaluate expressions with parentheses
fn evaluate_with_parentheses(expr: &str, use_degrees: bool) -> Result<f64> {
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
    evaluate_basic(&expr)
}

/// Evaluate basic arithmetic expression without parentheses
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

/// Evaluate a term (contains *, /, % operators)
fn evaluate_term(term: &str) -> Result<f64> {
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
    let mut result = validate_number(factors[0])?;
    let mut op_index = 0;
    for i in 0..chars.len() {
        if chars[i] == '*' {
            let next = validate_number(factors[op_index + 1])?;
            result *= next;
            op_index += 1;
        } else if chars[i] == '/' {
            let next = validate_number(factors[op_index + 1])?;
            if next == 0.0 {
                anyhow::bail!("Division by zero");
            }
            result /= next;
            op_index += 1;
        } else if chars[i] == '%' {
            let next = validate_number(factors[op_index + 1])?;
            result %= next;
            op_index += 1;
        } else if chars[i] == '^' {
            let next = validate_number(factors[op_index + 1])?;
            result = result.powf(next);
            op_index += 1;
        }
    }
    Ok(result)
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
        assert_eq!(result, 20160.0); // 8! / 2 = 40320 / 2 = 20160
    }
}
