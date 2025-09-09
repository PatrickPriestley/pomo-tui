# Implementation Plan: ADHD-Focused Pomodoro Terminal Application

**Branch**: `001-build-a-terminal` | **Date**: 2025-09-08 | **Spec**: [/specs/001-build-a-terminal/spec.md]
**Input**: Feature specification from `/specs/001-build-a-terminal/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
4. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
5. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, or `GEMINI.md` for Gemini CLI).
6. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
7. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
8. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
Build a terminal-based pomodoro timer and task management application specifically designed for ADHD users, featuring distraction-free interface, structured work sessions with visual progress indicators, integration with developer tools (git, GitHub, Jira, Slack), and comprehensive offline-first architecture using Rust and Ratatui for high-performance terminal UI.

## Technical Context
**Language/Version**: Rust 1.75+
**Primary Dependencies**: Ratatui (terminal UI), SQLite, Duration (high-precision timers), rodio (audio playback)
**Storage**: SQLite for local data persistence
**Testing**: cargo test
**Target Platform**: Cross-platform terminal (macOS, Linux, Windows)
**Project Type**: single - CLI application with modular architecture
**Performance Goals**: Instant startup (<50ms), 60fps UI updates, timer precision <100ms drift over 25 minutes
**Constraints**: <50MB memory usage, fully offline-capable, no external dependencies for core features
**Scale/Scope**: Single-user desktop application, ~10K LOC, 5-7 main screens (task list, timer, statistics, settings)

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Simplicity**:
- Projects: 2 (cli, tests)
- Using framework directly? Yes (Ratatui without wrappers)
- Single data model? Yes (direct SQLite entities)
- Avoiding patterns? Yes (no unnecessary abstractions)

**Architecture**:
- EVERY feature as library? Yes (modular architecture planned)
- Libraries listed:
  - `timer-core`: Timer logic and state management
  - `task-manager`: Task CRUD and priority management
  - `database`: SQLite persistence layer
  - `tui-renderer`: Ratatui UI components
  - `integrations`: Git, GitHub, Jira, Slack connectors
  - `audio`: Sound playback for ambient noise
  - `statistics`: Analytics and reporting
- CLI per library: Yes (each exposes --help/--version/--format)
- Library docs: llms.txt format planned? Yes

**Testing (NON-NEGOTIABLE)**:
- RED-GREEN-Refactor cycle enforced? Yes
- Git commits show tests before implementation? Yes
- Order: Contract→Integration→E2E→Unit strictly followed? Yes
- Real dependencies used? Yes (SQLite, actual file I/O)
- Integration tests for: new libraries, contract changes, shared schemas? Yes
- FORBIDDEN: Implementation before test, skipping RED phase - Acknowledged

**Observability**:
- Structured logging included? Yes (tracing crate)
- Frontend logs → backend? N/A (single CLI app)
- Error context sufficient? Yes (detailed error chains)

**Versioning**:
- Version number assigned? 0.1.0 (initial)
- BUILD increments on every change? Yes
- Breaking changes handled? Yes (migration scripts for DB)

## Project Structure

### Documentation (this feature)
```
specs/[###-feature]/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
# Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure]
```

**Structure Decision**: Option 1 (Single project) - CLI application with modular library structure

## Phase 0: Outline & Research
1. **Extract unknowns from Technical Context** above:
   - Website blocking mechanism in terminal environment
   - Audio playback integration with Rust
   - GitHub Issues sync behavior (read-only vs bidirectional)
   - Jira authentication and sync approach
   - Slack OAuth and status update mechanism
   - Export formats for data (CSV, JSON, etc.)
   - Session recovery after crashes

2. **Generate and dispatch research agents**:
   ```
   Task: "Research website blocking on macOS/Linux/Windows from Rust"
   Task: "Research audio playback libraries for Rust (rodio alternatives)"
   Task: "Research GitHub API integration patterns for Rust"
   Task: "Research Jira REST API authentication from CLI tools"
   Task: "Research Slack API OAuth flow for CLI applications"
   Task: "Best practices for Ratatui TUI architecture"
   Task: "SQLite schema migration patterns in Rust"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with all NEEDS CLARIFICATION resolved

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

1. **Extract entities from feature spec** → `data-model.md`:
   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:
   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Generate contract tests** from contracts:
   - One test file per endpoint
   - Assert request/response schemas
   - Tests must fail (no implementation yet)

4. **Extract test scenarios** from user stories:
   - Each story → integration test scenario
   - Quickstart test = story validation steps

5. **Update agent file incrementally** (O(1) operation):
   - Run `/scripts/update-agent-context.sh [claude|gemini|copilot]` for your AI assistant
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, agent-specific file

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
- Load `/templates/tasks-template.md` as base
- Generate tasks from Phase 1 design docs (contracts, data model, quickstart)
- Database schema and migrations first
- Each CLI command → contract test task [P]
- Each entity → model creation task [P]
- Each library module → separate implementation task
- Integration tests for cross-module communication
- UI components after business logic

**Specific Task Categories**:
1. **Setup Tasks** (1-5):
   - Initialize Rust project with dependencies
   - Set up SQLite with migrations
   - Configure logging and error handling

2. **Contract Test Tasks** (6-15):
   - One test per CLI endpoint from cli-api.yaml
   - Tests must fail initially (no implementation)

3. **Core Library Tasks** (16-25):
   - timer-core library with Duration precision
   - task-manager CRUD operations
   - database persistence layer
   - State machine for session/break transitions

4. **UI Tasks** (26-35):
   - Ratatui app structure
   - Task list view
   - Timer display with progress bar
   - Keyboard input handling

5. **Integration Tasks** (36-40):
   - Git commit enhancement
   - GitHub issue sync
   - Slack status updates
   - Website blocking

**Ordering Strategy**:
- TDD order: Tests before implementation
- Dependency order: Core → Services → UI → Integrations
- Mark [P] for parallel execution (independent files)

**Estimated Output**: 40-45 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |


## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented (none needed)

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*