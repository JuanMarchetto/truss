# Truss -- Goal, Architecture, and Guidelines

This is the document that explains what Truss is, how it's structured, and what rules we follow when building it. It's meant for anyone working on the project -- human or AI -- so we stay aligned on the big decisions and don't accidentally turn this into something it's not.

---

## 1. Project Goal

Truss is a CI/CD pipeline validator written in Rust. It catches configuration errors, semantic issues, and offers autocomplete -- all before you ever push and wait for a pipeline to fail.

The things we care about most:
- Speed and scalability
- Semantic correctness (we go deeper than syntax)
- A modular design you can actually reason about
- Measuring things with real benchmarks, not vibes

Truss is not a UI product or a SaaS platform. It's a core engine that can be plugged into different surfaces -- an LSP server, a CLI tool, a WASM build. The engine is the product.

---

## 2. Design Principles (Non-Negotiable)

These aren't suggestions. Every decision should pass through this filter:

1. **Core First**
   All the real logic lives in `truss-core`. The LSP, CLI, and WASM layers are adapters. They translate between the outside world and the core -- they don't contain business logic themselves.

2. **Performance Is a Feature**
   If something makes the code simpler but makes it slower, and you can't show numbers proving the tradeoff is worth it, don't do it.

3. **Measurable from Day One**
   If you can't benchmark it with `criterion` or `hyperfine`, you probably can't tell if you broke it.

4. **Incremental by Design**
   Assume the user is editing a file in real time. Parsing and validation should take advantage of partial updates, not reprocess everything from scratch.

5. **Developer Experience Matters**
   Error messages are part of the product. "Invalid configuration" is not helpful. Tell the developer what's wrong, where it is, and ideally how to fix it.

6. **Concurrency by Design**
   - Independent work should be parallelizable
   - Concurrency choices must be backed by measurements
   - Results must be deterministic regardless of execution order

---

## 3. Target Architecture (High Level)

```
VS Code Extension / Other Editors / CLI / WASM
        |
        v
 editors/vscode / truss-lsp / truss-cli / truss-wasm   (adapters)
        |
        v
      truss-core  (the engine)
        |
        +-- Parser (tree-sitter)
        +-- Incremental AST
        +-- Validation Engine (41 rules)
        +-- Schemas / Rules
```

### What each piece does:

- **truss-core**
  - Incrementally parses YAML via tree-sitter
  - Builds a semantic representation of the workflow
  - Runs validation rules against it
  - Exposes a stable, testable API that everything else depends on

- **truss-lsp**
  - Wraps `truss-core` in the LSP protocol
  - Manages open documents, versions, and diagnostics
  - This is a translation layer, not a logic layer

- **truss-cli**
  - Runs validations from the command line
  - Handles glob patterns, directory scanning, stdin, severity filtering, and JSON output
  - Also serves as the baseline for performance benchmarks

- **editors/vscode**
  - A VS Code extension that spawns `truss-lsp` over stdio
  - Activates on `.github/workflows/*.yml` files
  - Deliberately thin -- it's just an LSP client, no validation logic lives here

- **truss-wasm (future)**
  - Same core, compiled to WASM. No logic changes needed -- that's the whole point of the architecture.

---

## 4. Responsibility Boundaries

This is where coupling sneaks in if you're not careful:

- `truss-core` **must not**:
  - Know anything about LSP
  - Know anything about VS Code
  - Handle editor-specific I/O

- Adapters **must not**:
  - Implement validation rules
  - Duplicate logic that belongs in the core

If you're writing a validation rule in `truss-lsp`, stop. It goes in `truss-core`.

---

## 5. Development Guidelines (Humans and AI)

### 5.1 Before You Write Code

Ask yourself:
1. Does this belong in the core or in an adapter?
2. Can I measure its impact?
3. Does it affect performance?
4. Can I test it without spinning up an editor?

If any answer is "no" or "I'm not sure," step back and rethink the design before writing code.

---

### 5.2 Internal APIs

- Prefer pure functions. They're easier to test and harder to break.
- No global state. Seriously.
- Use explicit types and typed errors -- stringly-typed APIs are a liability.
- Document important invariants when they're not obvious from the types.

Example of a useful invariant comment:
> "This function assumes the AST is consistent with the latest incremental parse."

---

### 5.3 Optimization Rules

- Don't optimize without measuring first. Gut feelings are not benchmarks.
- Any meaningful optimization needs:
  - A benchmark before the change
  - A benchmark after the change
  - A note explaining what changed and why it's faster

---

## 6. Benchmarks Are First-Class Citizens

Benchmarks aren't a nice-to-have. They're how we know the project is working.

The rules:
- Unit benchmarks live next to their crate in `benches/`
- Comparative benchmarks (e.g., "how does Truss compare to actionlint") live in `truss-cli`
- Results should be exportable as Markdown or JSON

Any change that touches the core should consider its benchmark impact. If you're not sure, run them.

---

## 7. Current Scope (MVP)

What's in:
- GitHub Actions workflow validation
- Semantic checks (not just "is this valid YAML")
- Contextual autocompletion
- Incremental parsing

What's out (for now):
- Azure Pipelines, GitLab CI, etc.
- Any kind of advanced UI
- Complex multi-repo configuration
- Monetization

We'll get to those later. Right now the goal is a rock-solid core for GitHub Actions.

---

## 8. Final Rule (for Humans and AI)

> If a decision makes Truss faster, more predictable, and easier to reason about, it is probably correct.
> If it makes the system more magical, tightly coupled, or harder to measure, it is probably wrong.

This document should only be updated when the project's direction changes, not for implementation details.
