# Truss – Goal, Architecture and Guidelines

This document defines the **purpose, target architecture, and design guidelines** of the Truss project. It is intended to serve as a **single source of truth** for both human contributors and an **AI development assistant**, guiding technical decisions, preventing scope creep, and ensuring long-term coherence.

---

## 1. Project Goal

Truss is a CI/CD pipeline validation and analysis engine written in Rust, designed to provide **real-time feedback**, **high performance**, and **reproducible results**. Its primary goal is to improve developer experience by detecting configuration errors, semantic inconsistencies, and autocomplete opportunities **before a pipeline is executed**.

The project prioritizes:
- Performance and scalability
- Semantic correctness (not just syntax)
- Modular and reusable design
- Objective measurement through benchmarks

Truss is **not** initially a visual product or a SaaS platform; it is a **solid core engine** that can be integrated into multiple surfaces (LSP, CLI, WASM).

---

## 2. Design Principles (Non‑Negotiable)

These rules must guide all technical decisions:

1. **Core First**  
   All critical logic lives in `truss-core`. LSP, CLI, or WASM layers are adapters, not business-logic containers.

2. **Performance Is a Feature**  
   If a decision simplifies code but degrades performance without measurement, it must be rejected or explicitly justified with benchmarks.

3. **Measurable from Day One**  
   Every critical component must be benchmarkable using `criterion` or `hyperfine`.

4. **Incremental by Design**  
   The system must assume partial file edits. Parsing and validation should exploit this property.

5. **Developer Experience Matters**  
   Clear, actionable error messages are part of the product, not an afterthought.

6. **Concurrency by Design**  
   - Independent operations must be parallelizable
   - Concurrency decisions must be benchmarked and measured
   - Determinism must be maintained regardless of execution order

---

## 3. Target Architecture (High Level)

```
Editor / CLI / WASM
        │
        ▼
 truss-lsp / truss-cli / truss-wasm   (adapters)
        │
        ▼
      truss-core  (system core)
        │
        ├─ Parser (tree-sitter)
        ├─ Incremental AST
        ├─ Validation Engine
        └─ Schemas / Rules
```

### Clear responsibilities:

- **truss-core**
  - Incrementally parse YAML
  - Build a semantic representation
  - Execute validation rules
  - Expose a stable, testable API

- **truss-lsp**
  - Adapt `truss-core` to the LSP protocol
  - Handle documents, versions, and diagnostics

- **truss-cli**
  - Run validations from the command line
  - Act as the foundation for comparative benchmarks

- **truss-wasm (future)**
  - Reuse the core without logical changes

---

## 4. Responsibility Boundaries

To avoid unintended coupling:

- `truss-core` **MUST NOT**:
  - Be aware of LSP
  - Be aware of VS Code
  - Handle editor-specific IO

- Adapters **MUST NOT**:
  - Implement validation rules
  - Duplicate core logic

---

## 5. Development Guidelines (Humans and AI)

### 5.1 When Adding New Code

Before writing code, answer:
1. Does this belong in the core or in an adapter?
2. Is it measurable?
3. Does it impact performance?
4. Can it be tested without an editor?

If any answer is negative, reconsider the design.

---

### 5.2 Internal APIs

- Prefer pure functions
- Avoid global state
- Use explicit types and typed errors
- Document important invariants

Example:
> “This function assumes the AST is consistent with the latest incremental parse.”

---

### 5.3 Optimization Rules

- Do not optimize without measurement
- Any meaningful optimization must:
  - Have a benchmark before
  - Have a benchmark after
  - Be documented

---

## 6. Benchmarks as First‑Class Citizens

Benchmarks are not optional or secondary.

Rules:
- Unit benchmarks live alongside the crate (`benches/`)
- Comparative benchmarks live in `truss-cli`
- Results must be exportable (Markdown / JSON)

Any change affecting the core should consider its impact on existing benchmarks.

---

## 7. Current Scope (MVP)

Included:
- GitHub Actions
- Basic semantic validation
- Contextual autocompletion
- Incremental parsing

Excluded (for now):
- Azure Pipelines
- Advanced UI
- Complex configuration
- Monetization

---

## 8. Final Rule (for Humans and AI)

> If a decision makes Truss faster, more predictable, and easier to reason about, it is probably correct.
> If it makes the system more magical, tightly coupled, or harder to measure, it is probably wrong.

This document should only be updated when the **project goal changes**, not for implementation details.
