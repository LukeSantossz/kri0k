# Commit Message Convention

KRK-001 Kri0K follows [Conventional Commits](https://www.conventionalcommits.org/) v1.0.0.

## Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

## Types

- `feat`: New feature (user-facing or internal API)
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring (no feature change)
- `perf`: Performance improvement
- `test`: Adding or updating tests
- `build`: Build system or dependency changes
- `ci`: CI/CD configuration changes
- `chore`: Maintenance tasks (e.g., updating .gitignore)
- `revert`: Revert a previous commit

## Scopes (optional)

Scopes map to crates or modules:

- `core`: kri0k-core (runtime, validator, audit)
- `graph`: kri0k-graph (petgraph state)
- `ttp`: kri0k-ttp (TTP trait and implementations)
- `scope`: kri0k-scope (scope.yaml parser)
- `pybridge`: kri0k-pybridge (PyO3 bindings)
- `agent`: Python agent code (LangGraph, LLM providers)
- `cli`: Command-line interface
- `deps`: Dependency updates
- `adr`: Architecture Decision Records

## Subject

- Use imperative mood ("add" not "added" or "adds")
- Lowercase first letter
- No period at the end
- Max 72 characters

## Body (optional)

- Wrap at 72 characters
- Explain *what* and *why*, not *how*
- Reference issues/PRs: `Fixes #123`, `Relates to #456`

## Footer (optional)

- `BREAKING CHANGE:` for breaking changes (triggers major version bump)
- `Reviewed-by:` for code review attribution
- `Refs:` for related issues

## Examples

### Simple feature
```
feat(ttp): add T1046 network scanner TTP

Implements T1046 (Network Service Discovery) using nmap.
Includes validation against scope CIDR ranges.

Refs: #42
```

### Bug fix with breaking change
```
fix(graph)!: change Node.id to ULID format

BREAKING CHANGE: Node.id is now a ULID string instead of u32.
This ensures stable IDs across graph serialization/deserialization.

Migration: re-serialize existing graphs with the new format.

Fixes: #78
```

### Documentation update
```
docs(adr): add ADR-0013 for scope inheritance

Documents the decision to support YAML anchors in scope.yaml
for inheriting target lists between profiles.
```

### Dependency update
```
build(deps): update petgraph to 0.6.5

Updates petgraph to fix a correctness bug in StableGraph removal.
```

### Refactoring
```
refactor(pybridge): extract snapshot codec to separate module

No functional change. Improves code organization and testability.
```

## Tooling

### Commitizen (optional)
Install commitizen for interactive commit message prompts:

```bash
pip install commitizen
cz commit
```

### Pre-commit hook (enforced)
The pre-commit hook validates commit messages against Conventional Commits.

## Enforcement

- CI checks commit messages on PRs
- Squash merges must follow the convention
- Release notes are auto-generated from commit messages
