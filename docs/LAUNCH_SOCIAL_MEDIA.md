# Truss Launch - Social Media Publication Guide

This document contains ready-to-publish content for announcing Truss across relevant platforms, along with strategic timing and positioning guidance.

---

## Strategic Positioning

### Core Narrative
Truss follows the "Ruff playbook" — the same strategy that took the Python linter Ruff from zero to 33,000 GitHub stars and $4M in funding: **rewrite a beloved but slow tool in Rust and lead with benchmarks.**

### Key Differentiators
1. **15-35x faster** than actionlint (the current market leader)
2. **Semantic validation** — not just schema/syntax checking
3. **39 rules** — more comprehensive than any single competitor
4. **LSP server** — real-time editor diagnostics at 11.1ms
5. **Rust + tree-sitter** — the modern dev tools stack

### Competitive Landscape
| Tool | Language | Focus | Stars | Speed vs Truss |
|------|----------|-------|-------|----------------|
| actionlint | Go | General linting | 3.6k | 15x slower |
| zizmor | Rust | Security only | 3k | Different scope |
| action-validator | Rust | Schema only | ~200 | Different scope |
| **Truss** | **Rust** | **Full semantic** | **New** | **Baseline** |

### Market Context
- 6M+ daily GitHub Actions workflow runs
- 44% of GitHub repositories use Actions
- "Push and pray" debugging is the #1 pain point
- tj-actions supply chain attack (2024) affected 23,000 repos
- No tool combines speed + semantic depth + LSP integration

---

## Launch Sequence (Recommended Timeline)

| Day | Platform | Time (ET) | Priority |
|-----|----------|-----------|----------|
| Tuesday | Hacker News (Show HN) | 8:00-9:00 AM | Critical |
| Tuesday | Twitter/X | 10:00 AM | High |
| Tuesday | LinkedIn | 12:00 PM | High |
| Wednesday | Reddit (r/rust) | 9:00 AM | High |
| Wednesday | Reddit (r/github) | 11:00 AM | Medium |
| Thursday | Dev.to blog post | 9:00 AM | Medium |
| Friday | Reddit (r/devops) | 9:00 AM | Medium |

**Why Tuesday?** Hacker News engagement peaks Tuesday-Thursday mornings. Launching on Tuesday gives time to ride the wave across platforms.

---

## Platform 1: Hacker News (Show HN)

### Title
```
Show HN: Truss - A Rust-based GitHub Actions validator (15x faster than actionlint)
```

### URL
```
https://github.com/JuanMarchetto/truss
```

### First Comment (Post immediately after submission)

```
Hi HN, I built Truss because I was tired of the "push, wait 2 minutes,
see it fail, fix a typo, push again" cycle with GitHub Actions.

The existing tools either:
- Only validate syntax/schema (miss semantic bugs)
- Are too slow for real-time editor integration
- Don't catch the errors that actually waste your time

Truss is a Rust-based validator built on tree-sitter that runs 39 semantic
validation rules in ~11ms. That's fast enough to run on every keystroke
in your editor via LSP.

Some numbers from Hyperfine benchmarks:
- Truss: 11.1ms
- actionlint: 165.7ms (15x slower)
- yaml-language-server: 381.7ms (35x slower)

What it catches that others miss:
- Circular job dependencies
- Invalid matrix configurations (scalar instead of array)
- Undefined step output references (steps.X.outputs.Y)
- Secrets referenced but never defined in reusable workflows
- Runner label typos (e.g., "ubunty-latest")

The architecture is designed around the Ruff model: a core engine that's
editor-agnostic and deterministic, with thin adapters for CLI, LSP, and
WASM. All 39 rules run in parallel via rayon.

Current status: MVP complete with 257+ tests, working LSP server,
and CLI with parallel file processing. MIT licensed.

I'd appreciate any feedback on the rule coverage, architecture decisions,
or performance approach. Happy to answer questions.
```

---

## Platform 2: Twitter/X

### Launch Tweet

```
I built a GitHub Actions validator in Rust that's 15x faster than actionlint.

Truss validates workflows in 11ms — fast enough for real-time LSP feedback as you type.

39 semantic rules. 257+ tests. MIT licensed.

No more "push and pray" debugging.

github.com/JuanMarchetto/truss

#rustlang #opensource #githubactions
```

### Follow-up Thread (Post as replies)

**Tweet 2:**
```
Why build another linter?

Existing tools either:
- Only check syntax/schema
- Are too slow for editor integration
- Miss semantic bugs that actually waste your time

Truss catches circular dependencies, undefined outputs, invalid matrices, and 36 more rule categories.
```

**Tweet 3:**
```
Benchmark results (Hyperfine, complex workflow):

Truss (Rust): 11.1ms
actionlint (Go): 165.7ms — 15x slower
yaml-language-server (TS): 381.7ms — 35x slower
yamllint (Python): 210.9ms — 19x slower

Built on tree-sitter + rayon for parallel rule execution.
```

**Tweet 4:**
```
Architecture follows the Ruff model:

- Core engine: editor-agnostic, deterministic
- CLI: parallel file processing
- LSP: real-time diagnostics
- WASM: browser integration (coming)

All 39 rules are stateless and independently testable.

Feedback welcome — especially on rule coverage gaps.
```

---

## Platform 3: LinkedIn

### Post

```
I just open-sourced Truss, a GitHub Actions workflow validator I built in Rust.

The problem: GitHub Actions debugging is painful. You write a workflow, push it, wait for CI, see it fail on a typo, fix it, push again. Repeat. The average developer loses significant time in this "push and pray" cycle.

Existing tools help, but they're either limited to syntax checking or too slow for real-time use. I wanted something that could:

1. Run on every keystroke in an editor (requires <50ms latency)
2. Catch semantic bugs, not just syntax errors
3. Scale to hundreds of workflow files

So I built Truss.

The results:
- 15x faster than actionlint (the current Go-based market leader)
- 35x faster than yaml-language-server
- 39 semantic validation rules (circular dependencies, undefined references, invalid matrices, and more)
- 11.1ms average validation time on complex workflows
- Full LSP server for real-time editor diagnostics

The technical approach: tree-sitter for incremental YAML parsing, rayon for parallel rule execution, and a strict "core-first" architecture inspired by tools like Ruff (the Rust Python linter that went from 0 to 33k GitHub stars).

This is my first open-source project of this scale. I built it because I believe developer tools should be fast by default, not as an afterthought. Performance is a feature.

Currently at MVP: 257+ tests, CI/CD pipeline, and comprehensive benchmarking infrastructure. MIT licensed.

If you work with GitHub Actions, I'd genuinely appreciate your feedback on what rules are missing or what would make this useful for your workflow.

Link in comments.

#OpenSource #Rust #GitHubActions #DeveloperTools #DevOps
```

### First Comment
```
Link: https://github.com/JuanMarchetto/truss

Happy to discuss the architecture decisions or benchmark methodology.
```

---

## Platform 4: Reddit r/rust

### Title
```
[Show] Truss: GitHub Actions workflow validator — 15x faster than actionlint, built with tree-sitter + rayon
```

### Body

```
Hey r/rust,

I've been working on Truss, a GitHub Actions workflow validator built
in Rust. I wanted to share it here because the Rust ecosystem and
tooling philosophy heavily influenced the architecture.

**What it does:**
Validates GitHub Actions workflows beyond basic YAML syntax — it
catches semantic errors like circular job dependencies, undefined step
output references, invalid matrix configurations, and 36 more rule
categories.

**Performance:**
Hyperfine benchmark against complex dynamic workflow:
- **Truss**: 11.1ms
- actionlint (Go): 165.7ms (15x slower)
- yaml-language-server (TS): 381.7ms (35x slower)

**Stack:**
- `tree-sitter` for incremental YAML parsing
- `rayon` for parallel rule execution
- `clap` for CLI
- `tower-lsp` for LSP server
- `criterion` for benchmarking
- `serde` for JSON output

**Architecture:**
Following the "core-first" pattern — all validation logic is in
`truss-core` (editor-agnostic, deterministic). CLI, LSP, and WASM
are thin adapters. 39 rules are registered at startup and executed
in parallel via rayon.

**Current state:**
- 39 validation rules
- 257+ tests across 40 test files
- Working CLI with parallel file processing
- Working LSP server with incremental parsing
- Criterion + Hyperfine benchmarking infrastructure
- CI/CD pipeline (check, test, clippy, fmt)

**Repo:** https://github.com/JuanMarchetto/truss

I'm particularly interested in feedback on:
1. The tree-sitter usage patterns (lots of manual child-index
   navigation — is there a better way?)
2. The parallel validation approach with rayon
3. Any rules you wish existed for GitHub Actions

MIT licensed. Contributions welcome.
```

---

## Platform 5: Reddit r/devops

### Title
```
Built a GitHub Actions linter in Rust that catches semantic errors (not just syntax) — 15x faster than actionlint
```

### Body

```
If you've ever spent 20 minutes debugging a GitHub Actions workflow
only to find it was a typo in a step output reference or a circular
job dependency — this tool is for that.

**Truss** is a GitHub Actions workflow validator that goes beyond
YAML syntax checking:

- **Circular dependency detection** in job `needs`
- **Step output reference validation** (`steps.X.outputs.Y` —
  catches when X doesn't exist)
- **Matrix configuration errors** (scalar values where arrays
  are required)
- **Secret reference validation** in reusable workflows
- **Runner label validation** (catches "ubunty-latest" typos)
- **Expression syntax validation** (`${{ }}` blocks)
- **39 total rules** covering jobs, steps, triggers, permissions,
  and more

**Performance:** Validates a complex workflow in 11.1ms vs
actionlint's 165.7ms. Fast enough for real-time editor feedback
via LSP.

**Usage:**
```bash
# Validate a single file
truss validate .github/workflows/ci.yml

# Validate multiple files in parallel
truss validate .github/workflows/*.yml

# JSON output for CI integration
truss validate --json .github/workflows/ci.yml
```

**Repo:** https://github.com/JuanMarchetto/truss

Open source, MIT licensed. Feedback welcome — especially on what
rules would be most useful in your CI/CD workflows.
```

---

## Platform 6: Reddit r/github

### Title
```
Open-source GitHub Actions validator that catches semantic errors before you push (15x faster than actionlint)
```

### Body

```
I built Truss to eliminate the "push and pray" cycle with GitHub
Actions. Instead of pushing your workflow, waiting for CI, and
discovering you misspelled a step ID or created a circular dependency
— Truss catches these errors locally in 11ms.

**What makes it different from existing tools:**

1. **Semantic validation** — not just "is this valid YAML" but "will
   this workflow actually work". It checks job dependencies, step
   references, matrix configs, secret definitions, and more.

2. **Speed** — 15x faster than actionlint, 35x faster than
   yaml-language-server. Fast enough to run on every keystroke
   in your editor.

3. **LSP server** — Real-time diagnostics in any editor that
   supports the Language Server Protocol.

**39 validation rules** including:
- Circular job dependency detection
- Step output reference validation
- Matrix configuration errors
- Runner label typos
- Expression syntax validation
- Secret reference validation for reusable workflows
- And 33 more

**Repo:** https://github.com/JuanMarchetto/truss

Written in Rust, MIT licensed. Would love feedback from anyone who
works extensively with GitHub Actions workflows.
```

---

## Platform 7: Dev.to Blog Post

### Title
```
I Built a GitHub Actions Validator in Rust That's 15x Faster Than actionlint
```

### Tags
`rust, github, devops, opensource`

### Cover Image Suggestion
A terminal screenshot showing Truss validating a workflow in 11ms with green checkmark output.

### Body

```markdown
## The Problem

If you use GitHub Actions, you know the pain:

1. Write a workflow YAML file
2. Push to GitHub
3. Wait 1-3 minutes for CI to start
4. Discover you misspelled `ubuntu-latest` as `ubunty-latest`
5. Fix the typo, push again
6. Wait another 1-3 minutes
7. Discover a circular dependency between jobs
8. Repeat

This "push and pray" debugging cycle is one of the biggest
productivity killers in modern DevOps. And it's not just annoying —
misconfigured CI/CD pipelines cost organizations real money in
developer time and delayed deployments.

## Existing Solutions Fall Short

There are tools that help, but each has limitations:

- **actionlint** (Go) — The current leader with 3.6k GitHub stars.
  Good rule coverage but runs at ~166ms per file. Too slow for
  real-time editor integration.

- **yaml-language-server** (TypeScript) — Provides LSP support but
  at ~382ms per validation. Schema-focused rather than semantic.

- **action-validator** (Rust) — Fast but limited to schema
  validation. Doesn't catch semantic errors.

- **zizmor** (Rust) — Excellent but focused exclusively on security
  auditing, not general validation.

## Enter Truss

I built Truss to fill this gap: a tool that combines **speed**,
**semantic depth**, and **editor integration**.

### The Numbers

| Tool | Language | Mean Time | Relative Speed |
|------|----------|-----------|----------------|
| **Truss** | Rust | **11.1ms** | 1.00x (baseline) |
| actionlint | Go | 165.7ms | 15x slower |
| yaml-language-server | TypeScript | 381.7ms | 35x slower |
| yamllint | Python | 210.9ms | 19x slower |

### 39 Semantic Validation Rules

Truss doesn't just check if your YAML is valid. It checks if your
workflow will actually work:

**Job-level checks:**
- Circular dependency detection in `needs` graphs
- Duplicate job name detection
- Runner label validation (catches "ubunty-latest")
- Container configuration validation

**Step-level checks:**
- Step output reference validation (`steps.X.outputs.Y`)
- Unique step ID enforcement
- Shell type validation
- Timeout and working directory validation

**Workflow-level checks:**
- Reusable workflow input/output/secret validation
- Permission scope validation (15+ scopes)
- Concurrency group validation
- Event payload field validation (30+ event types)

**Expression checks:**
- `${{ }}` syntax validation
- Function validation (including `fromJSON`, `toJSON`, etc.)
- Action reference format validation (`owner/repo@ref`)

### Architecture: The Ruff Model

Truss follows the architecture pattern established by
[Ruff](https://github.com/astral-sh/ruff), the Rust Python linter
that went from zero to 33,000 GitHub stars:

1. **Core-first**: All validation logic lives in `truss-core`,
   which is editor-agnostic and fully deterministic.
2. **Thin adapters**: CLI, LSP, and WASM are separate crates that
   wrap the core.
3. **Parallel by default**: All 39 rules execute in parallel
   via rayon.
4. **Tree-sitter parsing**: Incremental parsing enables real-time
   editor integration without re-parsing the entire file.

### Quick Start

```bash
# Clone and build
git clone https://github.com/JuanMarchetto/truss.git
cd truss
cargo build --release

# Validate a workflow
./target/release/truss validate .github/workflows/ci.yml

# JSON output for CI integration
./target/release/truss validate --json .github/workflows/ci.yml
```

### LSP Integration

The LSP server provides real-time diagnostics in any editor:

```bash
# Run the LSP server
./target/release/truss-lsp
```

Configure your editor to use `truss-lsp` as the language server
for YAML files in `.github/workflows/`.

## What's Next

Truss is at MVP stage with a solid foundation:
- 257+ tests across 40 test files
- Comprehensive benchmarking infrastructure
- Clean CI/CD pipeline
- MIT licensed

Planned features:
- Contextual autocomplete
- WASM bindings for browser-based validation
- Directory/glob scanning
- Severity filtering

## Try It Out

**Repository:** [github.com/JuanMarchetto/truss](https://github.com/JuanMarchetto/truss)

I'd love feedback on:
- What validation rules are missing?
- What would make this useful for your workflow?
- Architecture suggestions from the Rust community

---

*Truss is open source and MIT licensed. Contributions welcome.*
```

---

## Platform 8: Product Hunt (Optional, Week 2)

### Tagline
```
Truss - GitHub Actions validator that's 15x faster than actionlint
```

### Description
```
Truss is a Rust-based GitHub Actions workflow validator that catches
semantic errors before you push. With 39 validation rules running in
11ms, it's fast enough for real-time editor integration via LSP.
Catches circular dependencies, undefined references, invalid matrices,
and more. Open source, MIT licensed.
```

### Topics
`Developer Tools`, `GitHub`, `Open Source`, `DevOps`, `Rust`

---

## Content Assets to Prepare Before Launch

### 1. Terminal Recording (Required for Twitter/HN)
Create a short (15-30 second) terminal recording showing:
```bash
# Show validation speed
time truss validate benchmarks/fixtures/complex-dynamic.yml

# Show error detection
truss validate examples/broken-workflow.yml
# (create a broken example that shows useful error output)
```

Tools: `asciinema` or `vhs` (the Charm tool for terminal GIFs)

### 2. Benchmark Chart Image (Required for Twitter/LinkedIn)
Create a horizontal bar chart showing:
- Truss: 11.1ms (green)
- actionlint: 165.7ms (gray)
- yamllint: 210.9ms (gray)
- yaml-language-server: 381.7ms (gray)

Tools: Use a simple charting tool or even a well-formatted screenshot.

### 3. Example Broken Workflow
Create a `.github/workflows/example-broken.yml` that demonstrates several
errors Truss catches, for use in demos and documentation:

```yaml
name: Broken CI
on: push
jobs:
  build:
    runs-on: ubunty-latest  # typo
    needs: [deploy]          # circular dependency
    steps:
      - uses: actions/checkout@v4
      - run: echo "test"
        id: test-step
      - run: echo ${{ steps.nonexistent.outputs.result }}  # undefined reference

  deploy:
    runs-on: ubuntu-latest
    needs: [build]           # circular dependency
    steps:
      - run: echo "deploy"
```

---

## Post-Launch Engagement Strategy

### Day 1-2: Active Engagement
- Respond to **every** HN comment within 1 hour
- Engage with Twitter replies and retweets
- Answer Reddit questions in detail
- Thank early stargazers

### Day 3-5: Follow-up Content
- Share interesting feedback/feature requests on Twitter
- Post a "Day 3 update" on HN if there's significant traction
- Cross-post the Dev.to article to personal blog/Medium

### Week 2: Sustain Momentum
- Submit to Product Hunt
- Write a "Lessons learned" follow-up post
- Engage with any blog posts or reviews from the community
- Open "good first issue" labels for contributors

### Ongoing
- Share benchmark updates when performance improves
- Announce new rules with brief technical explanations
- Highlight community contributions
- Monthly progress updates on Twitter/LinkedIn

---

## Metrics to Track

| Platform | Key Metric | Good Launch | Great Launch |
|----------|-----------|-------------|--------------|
| GitHub | Stars (Day 1) | 50+ | 200+ |
| GitHub | Stars (Week 1) | 200+ | 1,000+ |
| HN | Points | 50+ | 200+ |
| Twitter | Impressions | 10k+ | 50k+ |
| Reddit r/rust | Upvotes | 50+ | 200+ |
| Dev.to | Views | 1k+ | 5k+ |

---

## Key Messages Cheat Sheet

Use these consistently across all platforms:

- **Speed**: "15-35x faster than existing tools"
- **Latency**: "11ms — fast enough for real-time editor feedback"
- **Depth**: "39 semantic validation rules"
- **Pain point**: "No more push-and-pray debugging"
- **Architecture**: "Built on tree-sitter + rayon, following the Ruff model"
- **Status**: "MVP complete, 257+ tests, MIT licensed"
- **Ask**: "What rules are missing from your workflow?"
