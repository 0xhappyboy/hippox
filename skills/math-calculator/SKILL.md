---
name: math-calculator
description: Evaluate mathematical expressions
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "calculate"
    - "compute"
    - "what is"
    - "evaluate"
    - "solve"
  case_sensitive: false
parameters:
  - name: expression
    type: string
    description: Math expression to evaluate, e.g., "2 + 3 * 4" or "(10 - 5) / 2"
    required: true
  - name: precision
    type: integer
    description: Number of decimal places in result
    required: false
    default: 2
allowed_tools: []
metadata:
  emoji: 🧮
  category: math
---

# Calculator Skill

Evaluate mathematical expressions with support for basic arithmetic operations.

## Supported Operations

- Addition (`+`)
- Subtraction (`-`)
- Multiplication (`*`)
- Division (`/`)
- Modulo (`%`)
- Parentheses for grouping (`(` and `)`)

## Examples

**User:** "Calculate 15 * 3 - 8"
**Response:** 37.00

**User:** "What is (100 + 50) / 3?"
**Response:** 50.00

**User:** "Compute 2^10"
**Response:** 1024.00

## Instructions

1. Extract the mathematical expression from user input
2. Return the expression to be evaluated
3. Use the precision parameter to control decimal places (default 2)