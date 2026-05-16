---
name: system-systeminfo
description: Get system information like OS, CPU, memory, and disk usage
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "system info"
    - "system information"
    - "about this system"
    - "os info"
    - "cpu info"
    - "memory info"
    - "disk usage"
    - "system status"
  case_sensitive: false
parameters:
  - name: info_type
    type: string
    description: "Type of info: all, os, cpu, memory, disk, hostname"
    required: false
    default: all
allowed_tools: []
metadata:
  emoji: 💻
  category: system
---

# System Info Skill

Get information about the host system.

## Supported Info Types

| Type | Description |
|------|-------------|
| all | All system information |
| os | Operating system info |
| cpu | CPU information |
| memory | RAM usage |
| disk | Disk usage |
| hostname | System hostname |

## Examples

**User:** "Show me system information"
**Response:** OS: Linux, CPU: 8 cores, Memory: 15.6GB/32GB, Disk: 120GB/500GB

**User:** "How much RAM is available?"
**Response:** Available memory: 15.6 GB

**User:** "What is my hostname?"
**Response:** hostname: my-computer

## Instructions

1. Identify what system info user wants
2. Query system for requested information
3. Return formatted result