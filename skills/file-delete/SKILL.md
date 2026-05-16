---
name: file-delete
description: Delete a file or empty directory
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "delete file"
    - "remove file"
    - "delete directory"
    - "remove directory"
    - "rm"
    - "del"
  case_sensitive: false
parameters:
  - name: path
    type: string
    description: Path to the file or directory to delete
    required: true
  - name: recursive
    type: boolean
    description: Delete directory recursively (including all contents)
    required: false
    default: false
allowed_tools:
  - fs
metadata:
  emoji: 🗑️
  category: file
---

# File Delete Skill

Delete files or empty directories from the local filesystem.

## Features

- Delete individual files
- Delete empty directories
- Recursive deletion for non-empty directories (use with caution)

## Examples

**User:** "Delete /tmp/temp.txt"
**Response:** File deleted: /tmp/temp.txt

**User:** "Remove directory /tmp/old_folder and everything in it"
**Response:** Directory deleted recursively: /tmp/old_folder

## Instructions

1. Extract the path from user input
2. Check for recursive keywords ("recursive", "all contents", "everything in it")
3. Delete the specified file or directory