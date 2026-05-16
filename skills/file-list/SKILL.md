---
name: file-list
description: List contents of a directory
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "list files"
    - "list directory"
    - "show files"
    - "ls"
    - "dir"
    - "what's in"
    - "contents of"
  case_sensitive: false
parameters:
  - name: path
    type: string
    description: Directory path to list
    required: true
  - name: show_hidden
    type: boolean
    description: Show hidden files (starting with dot)
    required: false
    default: false
  - name: detail
    type: boolean
    description: Show detailed information (type, size)
    required: false
    default: false
allowed_tools:
  - fs
metadata:
  emoji: 📂
  category: file
---

# File List Skill

List contents of a directory on the local filesystem.

## Features

- List files and subdirectories
- Option to show hidden files
- Detailed view with file types and sizes

## Examples

**User:** "List files in /home/user"
**Response:** Contents of /home/user: file1.txt, file2.pdf, Documents

**User:** "Show detailed listing of /var/log"
**Response:** DIR logs, FILE system.log 1024 bytes, ...

## Instructions

1. Extract directory path from user input
2. Check for hidden files flag ("show hidden", "all files")
3. Check for detail flag ("detailed", "long format", "with details")
4. Return the directory listing