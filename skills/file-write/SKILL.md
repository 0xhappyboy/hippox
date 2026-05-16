---
name: file-write
description: Write content to a file
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "write file"
    - "save file"
    - "create file"
    - "write to file"
    - "save to file"
  case_sensitive: false
parameters:
  - name: path
    type: string
    description: Path to the file to write
    required: true
  - name: content
    type: string
    description: Content to write to the file
    required: true
  - name: append
    type: boolean
    description: Append to file instead of overwriting
    required: false
    default: false
allowed_tools:
  - fs
metadata:
  emoji: ✍️
  category: file
---

# File Write Skill

Write or append content to a file on the local filesystem.

## Features

- Create new files or overwrite existing ones
- Append mode to add content without overwriting
- Auto-create parent directories if needed

## Examples

**User:** "Save 'Hello World' to /tmp/hello.txt"
**Response:** Content written to file: /tmp/hello.txt

**User:** "Append 'new line' to log.txt"
**Response:** Content appended to file: log.txt

## Instructions

1. Extract file path and content from user input
2. Check if user wants to append (keywords like "append", "add to")
3. Write content to the specified file path