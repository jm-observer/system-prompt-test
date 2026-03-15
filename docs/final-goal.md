# Final Goal

## Project Positioning

Build a new web-based platform for testing and evaluating system prompts and model runtime behavior around tool, skill, and MCP calls.

## Core Product Goal

Provide a reproducible workflow to design prompts, run cases across providers/models, inspect full call traces, and verify expected behavior with rule-based assertions.

## Scope (Must Have)

1. Web project (UI + API backend).
2. System prompt management:
   - layered prompt composition (global/project/provider/model)
   - versioning, diff, rollback
   - variable injection
3. Provider management:
   - provider and model configuration
   - key management and masking
   - capability matrix (tool/stream/json/mcp)
4. Case and run management:
   - test case definitions
   - online execution and fixture replay
   - baseline comparison
5. Behavior validation:
   - assertions for must-call / must-not-call / whitelist / keyword triggers
   - pass/fail with evidence snippets
6. Trace and reporting:
   - end-to-end timeline (prompt -> output -> tool calls -> tool results)
   - run reports (latency, token usage, cost, failure categories)
   - exportable JSON report for CI
7. Security and compliance:
   - secret redaction (tokens, user ids, auth headers)
   - audit-friendly logs without leaking raw credentials

## Success Criteria

1. A new prompt version can be tested on multiple providers/models in one run.
2. Tool/skill/MCP behaviors are visible and explainable from trace data.
3. Regressions are automatically detected by assertions and baseline diff.
4. The same case can be replayed deterministically from saved fixtures.
5. Reports can be consumed by CI for release gates.

## Non-Goals (Initial Phase)

1. Building a full traffic MITM proxy in this new project.
2. Supporting every provider feature from day one.
3. Large-scale multi-tenant enterprise permissions in MVP.

