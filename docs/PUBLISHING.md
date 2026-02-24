# Publishing Guide

Step-by-step guide for making Truss public and publishing to crates.io and the VS Code Marketplace.

---

## Prerequisites

Before publishing anything:

- [ ] All tests pass (`cargo test --workspace`)
- [ ] Clippy is clean (`cargo clippy --workspace -- -D warnings`)
- [ ] Format is clean (`cargo fmt --all -- --check`)
- [ ] README.md is up to date
- [ ] LICENSE-MIT file exists at repo root
- [ ] All open PRs are merged or closed

---

## Step 1: Make the Repository Public

1. Go to https://github.com/JuanMarchetto/truss/settings
2. Scroll to **Danger Zone** → **Change repository visibility**
3. Change from Private → **Public**
4. Confirm the action

**Before making public, verify:**
- [ ] No secrets, tokens, or credentials in the codebase (`grep -r "ghp_" .` should return nothing outside CLAUDE.md)
- [ ] `.gitignore` covers sensitive files
- [ ] CLAUDE.md does not contain tokens (remove or redact the GitHub token before going public)
- [ ] LICENSE-MIT is present and correct

> **Important:** Remove or redact the GitHub API token from `CLAUDE.md` before making the repo public. Replace it with a placeholder like `YOUR_GITHUB_TOKEN`.

---

## Step 2: Publish `truss-core` to crates.io

`truss-core` must be published first because `truss-cli` depends on it.

### 2.1 Create a crates.io Account

1. Go to https://crates.io and log in with your GitHub account
2. Go to https://crates.io/settings/tokens
3. Create a new API token with **publish-update** scope
4. Run: `cargo login <your-token>`

### 2.2 Verify `truss-core/Cargo.toml`

Ensure these fields are present and correct:

```toml
[package]
name = "truss-core"
version = "0.1.0"
edition = "2021"
description = "High-performance GitHub Actions workflow validation engine"
license = "MIT"
repository = "https://github.com/JuanMarchetto/truss"
homepage = "https://github.com/JuanMarchetto/truss"
keywords = ["github-actions", "ci-cd", "validation", "linter", "yaml"]
categories = ["development-tools", "command-line-utilities"]
readme = "../../README.md"
```

### 2.3 Dry Run

```bash
cd crates/truss-core
cargo publish --dry-run
```

This validates the package without uploading. Fix any errors before proceeding.

### 2.4 Publish

```bash
cargo publish -p truss-core
```

Wait a few minutes for crates.io to index the new crate before publishing dependents.

---

## Step 3: Publish `truss-cli` to crates.io

### 3.1 Update Dependency

Once `truss-core` is live on crates.io, update `crates/truss-cli/Cargo.toml` to reference the published version instead of a path dependency:

```toml
[dependencies]
truss-core = "0.1.0"    # was: { path = "../truss-core" }
```

> **Alternative:** You can keep the path dependency for local development and let `cargo publish` resolve it automatically — Cargo replaces path dependencies with the published version if the version matches. Just make sure the version in `truss-core/Cargo.toml` matches.

### 3.2 Verify `truss-cli/Cargo.toml`

```toml
[package]
name = "truss-cli"
version = "0.1.0"
edition = "2021"
description = "CLI for Truss - high-performance GitHub Actions workflow validator"
license = "MIT"
repository = "https://github.com/JuanMarchetto/truss"
homepage = "https://github.com/JuanMarchetto/truss"
keywords = ["github-actions", "ci-cd", "validation", "linter", "cli"]
categories = ["development-tools", "command-line-utilities"]
readme = "../../README.md"

[[bin]]
name = "truss"
path = "src/main.rs"
```

### 3.3 Dry Run and Publish

```bash
cargo publish -p truss-cli --dry-run
cargo publish -p truss-cli
```

### 3.4 Verify Installation

```bash
cargo install truss-cli
truss --version
# Should print: truss 0.1.0
```

---

## Step 4: Publish the VS Code Extension

### 4.1 Create a Publisher Account

1. Go to https://marketplace.visualstudio.com/manage
2. Sign in with a Microsoft account
3. Create a **publisher** (e.g., `juanmarchetto`)
4. Note your publisher ID — it must match `publisher` in `editors/vscode/package.json`

### 4.2 Create a Personal Access Token (PAT)

1. Go to https://dev.azure.com → User Settings → Personal Access Tokens
2. Create a new token with:
   - **Organization:** All accessible organizations
   - **Scopes:** Marketplace → **Manage**
3. Copy the token

### 4.3 Install vsce

```bash
cd editors/vscode
npm install
npm install -g @vscode/vsce
```

### 4.4 Update package.json

Verify these fields match your publisher account:

```json
{
  "publisher": "juanmarchetto",
  "repository": {
    "type": "git",
    "url": "https://github.com/JuanMarchetto/truss"
  }
}
```

### 4.5 Add an Icon (Optional but Recommended)

Place a 128x128 PNG icon at `editors/vscode/icon.png` and reference it:

```json
{
  "icon": "icon.png"
}
```

### 4.6 Login and Publish

```bash
# Login with your PAT
vsce login juanmarchetto

# Package locally first to verify
vsce package
# This creates truss-validator-0.1.0.vsix

# Test the VSIX locally
code --install-extension truss-validator-0.1.0.vsix

# Publish to marketplace
vsce publish
```

### 4.7 Verify

After a few minutes, the extension should appear at:
`https://marketplace.visualstudio.com/items?itemName=juanmarchetto.truss-validator`

---

## Step 5: Post-Publishing Checklist

- [ ] `cargo install truss-cli` works from a clean machine
- [ ] `truss --version` prints the correct version
- [ ] VS Code extension installs from the marketplace
- [ ] Extension activates on `.github/workflows/*.yml` files
- [ ] Diagnostics appear in the editor (requires `truss-lsp` on PATH)
- [ ] crates.io pages render correctly (description, README, links)
- [ ] Marketplace page renders correctly
- [ ] GitHub repo is public and accessible
- [ ] CI badge is green

---

## Crates NOT Published

The following crates have `publish = false` and are intentionally excluded:

| Crate | Reason |
|-------|--------|
| `truss-lsp` | Distributed as a binary via the VS Code extension, not as a library |
| `truss-wasm` | Placeholder — not yet functional |

---

## Version Bumping (Future Releases)

When releasing a new version:

1. Update version in all `Cargo.toml` files that changed
2. Update `package.json` version for the VS Code extension
3. Tag the release: `git tag v0.2.0 && git push --tags`
4. Publish crates in order: `truss-core` first, then `truss-cli`
5. Publish VS Code extension: `cd editors/vscode && vsce publish`
6. Create a GitHub Release with changelog

### Semantic Versioning

- **Patch** (0.1.x): Bug fixes, minor improvements
- **Minor** (0.x.0): New validation rules, new CLI flags, non-breaking changes
- **Major** (x.0.0): Breaking API changes to `truss-core`

---

## Troubleshooting

### `cargo publish` fails with "no `description` set"
Add `description` to the crate's `Cargo.toml`.

### `cargo publish` fails with path dependency error
Ensure `truss-core` is already published and the version matches, or use `--allow-dirty` for testing.

### `vsce publish` fails with "publisher not found"
Verify the `publisher` field in `package.json` matches your marketplace publisher ID exactly.

### `vsce package` warns about missing repository
Add the `repository` field to `package.json`.

### Extension doesn't activate
Check that `truss-lsp` is on your PATH. Open VS Code's Output panel → select "Truss Language Server" to see logs.
