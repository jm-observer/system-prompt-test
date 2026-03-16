# Repository Guidelines

## Project Structure & Module Organization
This repository is currently lightweight, with contributor instructions centered on root-level documentation and configuration. Keep top-level files focused and easy to scan. If application code is added later, place source files under `src/`, tests under `tests/`, and static assets under `assets/` or `docs/` so the layout stays predictable.

Examples:
- `src/` for implementation code
- `tests/` for automated checks
- `docs/` for design notes or reference material

## Build, Test, and Development Commands
No project-specific build tooling is defined in the current repository state. When adding tooling, prefer standard entry points and document them here.

Typical commands to introduce:
- `npm test` or `pytest` to run the test suite
- `npm run lint` or `ruff check .` for static checks
- `npm run build` or `python -m build` for release artifacts

If you add a command, also add a short description in `README.md` or this file.

## Coding Style & Naming Conventions
Use consistent, readable formatting and keep changes narrowly scoped. Prefer 4 spaces for indentation in prose examples and Python, and repository-standard defaults for any formatter you introduce. Use descriptive names:
- `kebab-case` for Markdown filenames when appropriate
- `snake_case` for Python modules
- `PascalCase` for class names

Adopt automated formatting early (`prettier`, `ruff format`, or equivalent) rather than relying on manual cleanup.

## Testing Guidelines
Add tests alongside new behavior, not as a follow-up. Mirror the source layout under `tests/` and name test files after the unit under test, such as `tests/test_parser.py` or `src/foo.test.ts`. Prefer focused tests that cover expected behavior, edge cases, and regressions.

## Commit & Pull Request Guidelines
Keep commit messages short, imperative, and specific, such as `Add contributor guide` or `Document test layout`. Group related edits into a single commit when they serve one purpose.

Pull requests should include:
- A brief summary of what changed
- The reason for the change
- Testing notes
- Screenshots only if UI or rendered docs changed

## Documentation & Maintenance
Update this guide when the repository gains real build tooling, tests, or new top-level directories. Contributor documentation should reflect the actual project layout, not an aspirational one.
