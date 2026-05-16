---
name: file-read
description: Read content from a file
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "read file"
    - "open file"
    - "show file"
    - "display file"
    - "cat"
    - "view file"
    - "file content"
  case_sensitive: false
parameters:
  - name: path
    type: string
    description: Path to the file to read
    required: true
  - name: max_size
    type: integer
    description: Maximum bytes to read (default 1MB)
    required: false
    default: 1048576
allowed_tools:
  - fs
metadata:
  emoji: 📖
  category: file
---

# File Read Skill

Read content from a file on the local filesystem.

## Features

- Read text files
- Limit output size to prevent overwhelming responses
- Path traversal protection

## Examples

**User:** "Read the file /home/user/config.json"
**Response:** Returns the content of config.json

**User:** "Show me the first 1000 bytes of log.txt"
**Response:** Returns first 1000 bytes of log.txt

## Instructions

1. Extract the file path from user input
2. If user specifies size limit, use as max_size
3. Return the file content or error message