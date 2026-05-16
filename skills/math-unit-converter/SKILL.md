---
name: math-unit-converter
description: Convert between different units of measurement
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "convert"
    - "conversion"
    - "to"
    - "in"
    - "from"
    - "how many"
    - "change to"
  case_sensitive: false
parameters:
  - name: value
    type: string
    description: The numeric value to convert
    required: true
  - name: from
    type: string
    description: Source unit (m, km, cm, mm, mi, ft, in, meter, kilometer, mile, foot, inch)
    required: true
  - name: to
    type: string
    description: Target unit (m, km, cm, mm, mi, ft, in, meter, kilometer, mile, foot, inch)
    required: true
  - name: precision
    type: integer
    description: Number of decimal places in result
    required: false
    default: 2
allowed_tools: []
metadata:
  emoji: 📏
  category: math
---

# Unit Converter Skill

Convert between different units of measurement.

## Supported Units

### Length

| Unit       | Aliases                     |
| ---------- | --------------------------- |
| meter      | m, meter, meters            |
| kilometer  | km, kilometer, kilometers   |
| centimeter | cm, centimeter, centimeters |
| millimeter | mm, millimeter, millimeters |
| mile       | mi, mile, miles             |
| foot       | ft, foot, feet              |
| inch       | in, inch, inches            |

## Examples

**User:** "Convert 100 kilometers to miles"
**Response:** 100 km = 62.14 miles

**User:** "How many feet in 5 meters?"
**Response:** 5 m = 16.40 ft

**User:** "Convert 12 inches to centimeters"
**Response:** 12 in = 30.48 cm

## Instructions

1. Extract the numeric value from user input
2. Identify the source unit (from)
3. Identify the target unit (to)
4. Return the value, from unit, and to unit
5. Use precision to control decimal places (default 2)
