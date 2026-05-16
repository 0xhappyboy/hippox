---
name: time-datetime
description: Get current date and time, or perform timezone conversions
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "what time"
    - "current time"
    - "what date"
    - "today"
    - "now"
    - "date"
    - "timezone"
  case_sensitive: false
parameters:
  - name: timezone
    type: string
    description: Timezone like "Asia/Shanghai", "America/New_York", "UTC"
    required: false
  - name: format
    type: string
    description: Output format like "%Y-%m-%d %H:%M:%S"
    required: false
  - name: operation
    type: string
    description: "Operation type: now, utc, timestamp, or convert"
    required: false
    default: now
allowed_tools: []
metadata:
  emoji: 🕐
  category: time
---

# DateTime Skill

Get current date and time, convert between timezones, and get timestamps.

## Supported Operations

| Operation | Description |
|-----------|-------------|
| now | Current local time |
| utc | Current UTC time |
| timestamp | Unix timestamp |
| convert | Convert between timezones |

## Examples

**User:** "What time is it?"
**Response:** 2024-01-15 14:30:25

**User:** "Current time in Tokyo"
**Response:** 2024-01-15 23:30:25

**User:** "Unix timestamp now"
**Response:** 1705314625

## Instructions

1. Detect what time information user wants
2. If timezone mentioned, convert to that timezone
3. Return formatted datetime string