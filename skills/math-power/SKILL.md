---
name: math-power
description: Calculate power, square root, or cube root operations
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "power"
    - "square"
    - "square root"
    - "sqrt"
    - "cube"
    - "cube root"
    - "cbrt"
    - "^"
    - "to the power"
  case_sensitive: false
parameters:
  - name: base
    type: string
    description: Base number for power operation
    required: false
  - name: exponent
    type: string
    description: Exponent for power operation
    required: false
  - name: sqrt
    type: string
    description: Number to calculate square root (alternative to base+exponent)
    required: false
  - name: cbrt
    type: string
    description: Number to calculate cube root (alternative to base+exponent)
    required: false
  - name: precision
    type: integer
    description: Number of decimal places in result
    required: false
    default: 2
allowed_tools: []
metadata:
  emoji: 🔢
  category: math
---

# Power Skill

Calculate powers, square roots, and cube roots.

## Supported Operations

- Power: base ^ exponent
- Square root: √x
- Cube root: ∛x

## Examples

**User:** "What is 2 to the power of 10?"
**Response:** 2 ^ 10 = 1024.00

**User:** "Calculate square root of 16"
**Response:** √16 = 4.00

**User:** "Cube root of 27"
**Response:** ∛27 = 3.00

## Instructions

1. Identify the operation type from user input
2. For power: extract base and exponent
3. For sqrt: extract the number under square root
4. For cbrt: extract the number under cube root
5. Return the appropriate parameters