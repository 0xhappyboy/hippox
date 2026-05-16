---
name: net-httprequest
description: Send HTTP requests to web APIs and fetch data
version: 1.0.0
author: Hippox
triggers:
  patterns:
    - "http request"
    - "fetch"
    - "get api"
    - "call api"
    - "web request"
    - "post request"
  case_sensitive: false
parameters:
  - name: url
    type: string
    description: HTTP URL to request
    required: true
  - name: method
    type: string
    description: HTTP method (GET, POST, PUT, DELETE)
    required: false
    default: GET
  - name: headers
    type: object
    description: HTTP headers as key-value pairs
    required: false
  - name: body
    type: string
    description: Request body (for POST/PUT)
    required: false
  - name: timeout
    type: integer
    description: Request timeout in seconds
    required: false
    default: 30
allowed_tools:
  - http
metadata:
  emoji: 🌐
  category: net
---

# HTTP Request Skill

Send HTTP requests to web APIs and return the response.

## Supported Methods

- GET - Retrieve data
- POST - Submit data
- PUT - Update data
- DELETE - Remove data

## Examples

**User:** "Fetch https://api.github.com/repos/rust-lang/rust"
**Response:** Returns JSON response from GitHub API

**User:** "POST to https://httpbin.org/post with body {'name': 'test'}"
**Response:** Returns server response

## Instructions

1. Extract URL from user input
2. Identify HTTP method (default GET)
3. Extract headers and body if present
4. Execute request and return response