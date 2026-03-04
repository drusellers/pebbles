---
id: 1dut
status: draft
priority: medium
changelog_type: null
parent: null
blocked_by: []
tags: []
created_at: 2026-02-22T20:46:34.043423Z
updated_at: 2026-02-22T20:46:34.043423Z
timer_start: null
accumulated_duration_secs: 0
---

# Provide Answers Workflow

## Goal

If a plan has a list of questions, then this is a way for the human to provide answers to the LLM.

```sh
pebbles answer <IDish>
```

the idea is that we read the questions out to the user, and let them provide answers.

Starting with just one pass at a time. Present X questions, the user responds, we save that, update the plan.
generate new questions if needed.

## Acceptance Criteria

- [ ] Reads the question section, and present it to the user
- [ ] Reads in user answers, updates the plan, and then writes any new questions
- [ ] saves and exits

## Context

!agent: Scan the codebase and document relevant files, patterns, and constraints here.

## Implementation Checklist

!agent: Break down the implementation into steps and fill out this checklist
- [ ] Step 1: Description
- [ ] Step 2: Description
- [ ] Step 3: Description

## Questions

!agent: Identify any open questions or ambiguities that need clarification
- Question 1?
- Question 2?

## Log

!agent: Update this log as you work on this change, recording significant progress, decisions, and blockers
- YYYY-MM-DD: Change created

## Changelog Type

To be determined during implementation (feature/fix/change/deprecated/removed/security/internal)

## Events

1. 2026-02-22T20:48:36.273906+00:00 [rw1w] created
   {"parent":null,"priority":"medium","title":"Provide Answers Workflow"}


