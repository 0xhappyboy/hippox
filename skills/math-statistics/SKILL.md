---
name: math-statistics
description: Calculate statistical values from a set of numbers
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "statistics"
    - "average"
    - "mean"
    - "median"
    - "mode"
    - "sum"
    - "min"
    - "max"
    - "statistical"
  case_sensitive: false
parameters:
  - name: numbers
    type: array
    description: Array of numbers to analyze, e.g., [1, 2, 3, 4, 5]
    required: true
  - name: operation
    type: string
    description: Statistical operation (sum, mean, average, min, max, median, mode)
    required: true
  - name: precision
    type: integer
    description: Number of decimal places in result
    required: false
    default: 2
allowed_tools: []
metadata:
  emoji: 📊
  category: math
---

# Statistics Skill

Calculate various statistical measures from a list of numbers.

## Supported Operations

| Operation | Description |
|-----------|-------------|
| sum | Sum of all numbers |
| mean / average | Arithmetic mean |
| min | Minimum value |
| max | Maximum value |
| median | Middle value |
| mode | Most frequent value(s) |

## Examples

**User:** "Calculate the average of 10, 20, 30, 40, 50"
**Response:** mean = 30.00

**User:** "What's the median of [5, 2, 9, 1, 7]?"
**Response:** median = 5.00

**User:** "Find sum, min, and max of these numbers: 3, 7, 2, 9"
**Response:** sum = 21.00, min = 2.00, max = 9.00

## Instructions

1. Extract the list of numbers from user input
2. Identify the requested statistical operation
3. Return the numbers as an array and the operation
4. Use precision to control decimal places (default 2)