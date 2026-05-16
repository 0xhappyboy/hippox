---
name: email-composer
description: Professional email composition assistant for various scenarios
version: 1.0.0
author: Hippox
license: MIT
compatibility: ">=1.0.0"
triggers:
  patterns:
    - "write email"
    - "compose email"
    - "draft email"
    - "send email to"
    - "email template"
    - "professional email"
    - "follow up email"
    - "outreach"
  case_sensitive: false
allowed_tools:
  - clipboard
  - template
  - grammar_check
dependencies: []
metadata:
  author: Productivity Team
  version: 1.0.0
  emoji: ✉️
  os:
    - linux
    - macos
    - windows
  requires:
    api_key: false
parameters:
  - name: recipient
    type: string
    description: Email recipient name or role
    required: false
    default: "[Recipient]"
  - name: purpose
    type: string
    description: Email purpose (request, thank you, follow up, introduction, complaint, etc.)
    required: true
  - name: tone
    type: string
    description: Email tone
    required: false
    default: "professional"
  - name: details
    type: string
    description: Key details to include in the email
    required: false
    default: ""
  - name: length
    type: string
    description: Email length preference
    required: false
    default: "medium"
---

# Email Composer Skill

This skill helps users write professional, effective emails for any business scenario. It follows email best practices and adapts to different tones and purposes.

## Core Capabilities

1. **Multi-purpose Support** - Requests, introductions, follow-ups, thank you notes, complaints, resignations, etc.
2. **Tone Adaptation** - Professional, friendly, formal, persuasive, apologetic
3. **Structure Optimization** - Clear subject lines, proper greetings, logical flow, strong closings
4. **Grammar & Style Check** - Polished language, correct punctuation, active voice
5. **Personalization** - Incorporates recipient details and context

## Workflow

### Step 1: Understand Requirements

Extract from user input:

- Email purpose (required)
- Recipient information (name, role, relationship)
- Key points to communicate
- Desired outcome (action from recipient)
- Urgency level
- Relationship context (first time, existing client, colleague, manager)

### Step 2: Determine Structure

Based on purpose, use appropriate structure:

#### Request Email

- Polite opening
- Context/background
- Clear request with specifics
- Reason for request
- Flexible deadline
- Gratitude

#### Follow-up Email

- Reference to previous interaction
- Gentle reminder
- Additional value/information
- Clear next step
- Appreciation for time

#### Introduction Email

- Connection context (who referred)
- Brief intro of both parties
- Reason for introduction
- Suggested action
- Offer to facilitate

#### Thank You Email

- Specific gratitude
- Impact of their help
- Optional: next steps
- Warm closing

#### Complaint Email

- Factual problem description
- Impact on you/team
- Desired resolution
- Request for action
- Professional tone (no hostility)

### Step 3: Apply Tone Rules

**Professional Tone:**

- Use "Dear" greeting
- Formal language (avoid contractions)
- Complete sentences
- Respectful closings ("Sincerely", "Best regards")

**Friendly Tone:**

- Use "Hi" or "Hello"
- Natural conversation style
- Occasional exclamation points
- Warm closings ("Best", "Cheers", "Thanks")

**Persuasive Tone:**

- Strong subject line
- Benefit-focused language
- Social proof or data
- Clear call to action
- Scarcity/urgency (if appropriate)

**Apologetic Tone:**

- Immediate acknowledgment
- Sincere apology without overdoing
- Explanation (not excuse)
- Action plan to fix
- Assurance for future

### Step 4: Optimize Length

**Short** (50-100 words):

- Direct to point
- No fluff or pleasantries
- 2-3 sentences max

**Medium** (100-200 words):

- Brief context
- Main message
- Clear action

**Long** (200-350 words):

- Detailed background
- Multiple points or questions
- Comprehensive context

### Step 5: Generate Email

Output format:

```markdown
**Subject:** [Compelling subject line]

[Greeting],

[Body paragraphs]

[Closing],
[Your Name/Signature]

---

**Key Notes:**

- [Important reminders for the user]
- [Customization tips]
```
