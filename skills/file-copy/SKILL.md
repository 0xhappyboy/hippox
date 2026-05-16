---
name: file-copy
description: Copy or move a file
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "copy file"
    - "move file"
    - "rename file"
    - "cp"
    - "mv"
    - "duplicate file"
  case_sensitive: false
parameters:
  - name: source
    type: string
    description: Source file path
    required: true
  - name: destination
    type: string
    description: Destination file path
    required: true
  - name: move
    type: boolean
    description: Move instead of copy (rename/move)
    required: false
    default: false
allowed_tools:
  - fs
metadata:
  emoji: 🔄
  category: file
---

# File Copy/Move Skill

Copy or move files to new locations on the local filesystem.

## Features

- Copy files to new location
- Move files (rename or relocate)
- Auto-create destination directory if needed

## Examples

**User:** "Copy file.txt to /backup/file.txt"
**Response:** Copied file.txt to /backup/file.txt

**User:** "Move report.pdf to /archive/report.pdf"
**Response:** Moved report.pdf to /archive/report.pdf

## Instructions

1. Extract source and destination paths from user input
2. Check for move keywords ("move", "rename", "mv")
3. Perform copy or move operation
