# Identity Architecture Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename the portable identity identifier to `IdentitySlug` everywhere and deepen identity ownership under `src/identity/` without changing runtime behavior.

**Architecture:** The identity module becomes a directory-root module containing slug/entity behavior in `mod.rs` and details behavior in `details.rs`. All public and internal identifier interfaces use `slug`; the saved auth entity remains an `Identity`. Dynamic JSON parsing and current CLI enrichment behavior remain unchanged for the next independent change.

**Tech Stack:** Rust 2024, Cargo features, Clap, Base64URL, `serde_json::Value`

**Spec:** `docs/superpowers/specs/2026-08-28-identity-architecture-cleanup-design.md`

## Global Constraints

- Preserve the user's uncommitted `serde` dependency changes in `Cargo.toml` and `Cargo.lock`; they belong to the next parsing change.
- Do not replace `serde_json::Value` or derive serde types in this change.
- Do not change malformed-details classification or the CLI's current details-error suppression.
- Do not add identity details to `capture`, `use`, or `detach`.
- Do not retain aliases, deprecated names, conversion shims, or duplicate fields.
- Keep existing user-facing command messages unchanged; only generated positional help changes from `<IDENTITY>` to `<SLUG>`.
- Use `mod.rs` for the identity module root.
- Keep slug-to-file mapping private; add no storage interface or adapter.

---

### Task 1: Deepen the identity module without renaming interfaces

**Files:**
- Create: `src/identity/mod.rs`
- Create: `src/identity/details.rs`
- Delete: `src/identity.rs`
- Delete: `src/identity_details.rs`
- Modify: `src/lib.rs:10-19`
- Modify: `src/manager.rs:7-16, 103-120`

**Interfaces:**
- Consumes: existing `Identity`, `IdentityName`, `IdentityDetails`, `Identity::read_details`, and `CodexAuthManager::read_active_auth_details` interfaces.
- Produces: the same public interfaces, now owned by `identity`; crate-private `IdentityDetails::read_from(&Path) -> Result<Option<Self>, Error>` shared by `Identity` and `CodexAuthManager`.

- [ ] **Step 1: Record the behavior-preserving baseline**

Run:

```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: 15 tests pass and Clippy exits 0. This task changes module ownership only, so the existing interface tests are the test surface; no source-layout test should be added.

- [ ] **Step 2: Move the identity implementation into the directory-root module**

Create `src/identity/mod.rs` from the complete current contents of `src/identity.rs`, then add the gated child declaration and re-export directly after imports:

```rust
#[cfg(feature = "identity-details")]
mod details;

#[cfg(feature = "identity-details")]
pub use details::IdentityDetails;
```

Create `src/identity/details.rs` from the complete current contents of `src/identity_details.rs`. Its imports become:

```rust
use std::{fmt, fs, path::Path};

use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use serde_json::Value;

use super::Identity;
use crate::Error;
```

Delete `src/identity.rs` and `src/identity_details.rs` in the same patch so Rust sees only the new module layout.

- [ ] **Step 3: Replace the free reader with type-owned crate behavior**

In `src/identity/details.rs`, replace `read_auth_details` with:

```rust
impl IdentityDetails {
    pub(crate) fn read_from(path: &Path) -> Result<Option<Self>, Error> {
        let auth = fs::read(path).map_err(|source| Error::Io {
            action: "read auth details",
            path: path.to_path_buf(),
            source,
        })?;
        Ok(parse_auth_details(&auth))
    }
}
```

Keep `Identity::read_details` public and delegate to the type-owned reader:

```rust
pub fn read_details(&self) -> Result<Option<IdentityDetails>, Error> {
    IdentityDetails::read_from(&self.path)
}
```

- [ ] **Step 4: Update module consumers**

In `src/lib.rs`, remove the top-level `identity_details` module and export. Retain:

```rust
mod identity;

pub use identity::{Identity, IdentityName};
#[cfg(feature = "identity-details")]
pub use identity::IdentityDetails;
```

In `src/manager.rs`, remove the import of `identity_details::read_auth_details`, keep the gated `IdentityDetails` import, and change the active read to:

```rust
AuthStatus::Native | AuthStatus::Managed { .. } => {
    IdentityDetails::read_from(&self.auth_path())
}
```

- [ ] **Step 5: Verify the module move**

Run:

```bash
cargo fmt --all
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
```

Expected: 15 tests pass, Clippy exits 0, and diff check is empty.

- [ ] **Step 6: Commit the module move**

```bash
git add src/identity src/lib.rs src/manager.rs src/identity.rs src/identity_details.rs
git commit -m "refactor: nest identity details module"
```

---

### Task 2: Rename the identity identifier interface to slug

**Files:**
- Modify: `src/identity/mod.rs`
- Modify: `src/error.rs`
- Modify: `src/status.rs`
- Modify: `src/manager.rs`
- Modify: `src/cli.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: the nested identity module produced by Task 1.
- Produces: `IdentitySlug`, `Identity.slug`, slug fields in status/errors, and slug parameters throughout the manager and CLI. Removes `IdentityName` completely.

- [ ] **Step 1: Change the identity validation tests to the desired interface**

In `src/identity/mod.rs`, change the test imports and assertions first:

```rust
use crate::{Error, IdentitySlug};

#[test]
fn identity_slug_validation_accepts_portable_slugs() {
    for slug in ["personal", "OpenAI-Work", "org.dev", "test_2", "work.json"] {
        assert!(IdentitySlug::try_from(slug).is_ok());
    }
}

#[test]
fn identity_slug_validation_rejects_paths_and_shellish_slugs() {
    for slug in ["", "-prod", "my work", "../auth", "work/main", "work\\main"] {
        assert!(matches!(
            IdentitySlug::try_from(slug),
            Err(Error::InvalidIdentitySlug { .. })
        ));
    }
}
```

Run:

```bash
cargo test identity_slug_validation
```

Expected: compilation fails because `IdentitySlug` and `InvalidIdentitySlug` do not exist yet.

- [ ] **Step 2: Rename the core identity interface**

In `src/identity/mod.rs`, rename the type, implementations, validator, and entity field:

```rust
/// A valid identity slug.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentitySlug(String);

/// An identity entry in the manager directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// Portable user-defined identity identifier.
    pub slug: IdentitySlug,
    /// Storage path.
    pub path: PathBuf,
    /// Whether this identity is selected by `auth.json`.
    pub active: bool,
    /// Whether this identity entry exists but is unusable.
    pub broken: bool,
}
```

Rename `as_str`, `TryFrom`, `FromStr`, `AsRef<str>`, and `Display` implementations to `IdentitySlug`. Rename `validate_identity_name` to `validate_identity_slug`; construct `Error::InvalidIdentitySlug { slug: value.to_owned() }` on rejection.

- [ ] **Step 3: Rename errors and status fields**

In `src/error.rs`, import `IdentitySlug` and use these interfaces:

```rust
InvalidIdentitySlug { slug: String },
IdentityNotFound { slug: IdentitySlug },
IdentityAlreadyExists { slug: IdentitySlug },
IdentityBroken { slug: IdentitySlug },
```

Preserve entity-oriented messages while naming the invalid primitive precisely:

```text
invalid identity slug: {slug}
identity not found: {slug}
identity already exists: {slug}
identity is broken: {slug}
```

In `src/status.rs`, import `IdentitySlug` and rename managed fields:

```rust
Managed { slug: IdentitySlug },
BrokenManaged { slug: IdentitySlug },
```

Rename `SymlinkTargetHasInvalidIdentityName` to `SymlinkTargetHasInvalidIdentitySlug` and render `symlink target has invalid identity slug`.

- [ ] **Step 4: Migrate the manager implementation and tests**

In `src/manager.rs`:

- Import `IdentitySlug` instead of `IdentityName`.
- Rename every identifier-only parameter and local from `identity` or `name` to `slug`.
- Construct `Identity { slug, path, active, broken }`.
- Sort and compare using `identity.slug`.
- Use status patterns `AuthStatus::Managed { slug }` and `BrokenManaged { slug }`.
- Use error fields `{ slug }` and reason `SymlinkTargetHasInvalidIdentitySlug`.
- Keep method names `capture`, `use_identity`, and `identity_path`; those methods act on or locate an Identity.

The manager interfaces become:

```rust
pub fn capture(&self, slug: &IdentitySlug, options: CaptureOptions) -> Result<(), Error>;
pub fn use_identity(&self, slug: &IdentitySlug, options: UseOptions) -> Result<(), Error>;
fn identity_path(&self, slug: &IdentitySlug) -> PathBuf;
fn relative_identity_path(slug: &IdentitySlug) -> PathBuf;
fn require_usable_identity(&self, slug: &IdentitySlug) -> Result<(), Error>;
```

Update all manager tests to construct `IdentitySlug` and assert slug fields.

- [ ] **Step 5: Migrate the CLI and crate exports**

In `src/lib.rs`, export:

```rust
pub use identity::{Identity, IdentitySlug};
```

In `src/cli.rs`:

- Import `IdentitySlug`.
- Rename `Command::Capture { identity, .. }` and `Command::Use { identity, .. }` fields to `slug`.
- Call manager methods with `&slug`.
- Keep messages such as `Captured identity: {slug}` and `Active identity: {slug}`.
- Render and complete identities through `identity.slug`.
- Update tests to construct `Identity { slug: IdentitySlug::try_from("personal").unwrap(), path, active, broken }` and managed status values with `slug: IdentitySlug::try_from("personal").unwrap()`.

The Clap variants should read:

```rust
Capture {
    /// Identity slug to create or overwrite.
    #[arg(add = ArgValueCompleter::new(identity_completer))]
    slug: IdentitySlug,
    #[arg(long)]
    force: bool,
},
Use {
    /// Identity slug to activate.
    #[arg(add = ArgValueCompleter::new(identity_completer))]
    slug: IdentitySlug,
    #[arg(long)]
    force: bool,
},
```

- [ ] **Step 6: Verify the breaking rename**

Run:

```bash
cargo fmt --all
cargo test
cargo test --features identity-details
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
rg -n "IdentityName|InvalidIdentityName|SymlinkTargetHasInvalidIdentityName|identity\.name" src
git diff --check
```

Expected: default tests pass; 13 library tests pass with `identity-details`; 15 tests pass with all features; Clippy exits 0; `rg` returns no matches; diff check is empty.

- [ ] **Step 7: Commit the breaking slug rename**

```bash
git add src
git commit -m "refactor!: rename identity names to slugs" -m "BREAKING CHANGE: IdentityName is replaced by IdentitySlug, and identity, status, and error identifier fields are now named slug."
```

---

### Task 3: Rename the account detail field to name

**Files:**
- Modify: `src/identity/details.rs`
- Modify: `src/manager.rs`
- Modify: `src/cli.rs`

**Interfaces:**
- Consumes: `IdentityDetails { display_name, email }` from Task 1.
- Produces: `IdentityDetails { name, email }`; removes `display_name` completely while preserving `Display` output.

- [ ] **Step 1: Change details tests to the desired field**

Update test construction and assertions in `src/identity/details.rs`, `src/manager.rs`, and `src/cli.rs` from `display_name` to `name`, for example:

```rust
IdentityDetails {
    name: Some("Example User".to_owned()),
    email: Some("the.user@gmail.com".to_owned()),
}
```

Run:

```bash
cargo test --all-features details
```

Expected: compilation fails because `IdentityDetails` still has `display_name`.

- [ ] **Step 2: Rename the production field and parser assignment**

In `src/identity/details.rs`, change the public type and all owned behavior:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityDetails {
    /// Account display name.
    pub name: Option<String>,
    /// Account email address.
    pub email: Option<String>,
}
```

Update `Display`, `parse_auth_details`, and the at-least-one-claim check to use `name`. Dynamic lookup remains:

```rust
let details = IdentityDetails {
    name: clean_claim(claims.get("name")),
    email: clean_claim(claims.get("email")),
};
(details.name.is_some() || details.email.is_some()).then_some(details)
```

- [ ] **Step 3: Verify the details-field rename**

Run:

```bash
cargo fmt --all
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
rg -n "display_name" src
git diff --check
```

Expected: 15 tests pass, Clippy exits 0, `rg` returns no matches, and diff check is empty.

- [ ] **Step 4: Commit the breaking details rename**

```bash
git add src/identity/details.rs src/manager.rs src/cli.rs
git commit -m "refactor!: rename identity detail name" -m "BREAKING CHANGE: IdentityDetails.display_name is replaced by IdentityDetails.name."
```

---

### Task 4: Align domain and user documentation

**Files:**
- Modify: `CONTEXT.md`
- Modify: `docs/adr/0001-use-identity-files-and-auth-json-symlink.md`
- Modify: `README.md`

**Interfaces:**
- Consumes: the implemented `IdentitySlug` terminology.
- Produces: one documented meaning for Identity and Identity slug; no code changes.

- [ ] **Step 1: Update the domain glossary**

Replace the **Identity name** entry in `CONTEXT.md` with:

```markdown
**Identity slug**:
The user-defined portable identifier of an **Identity**. CAM stores each identity as `<identity slug>.json`, so `.json` can be part of the identity slug.
_Avoid_: Filename, path, profile name, identity name
```

Correct the Native auth file definition from “before CAM captures it into an Auth slot” to “before CAM captures it into an Identity.” Update dialogue only where it refers specifically to the identifier rather than the Identity entity.

- [ ] **Step 2: Update ADR-0001 and README storage terminology**

In ADR-0001, describe storage as:

```markdown
CAM stores each identity as `$CODEX_HOME/codex-auth-manager/<identity slug>.json`.
```

Retain Identity as the user-facing entity and retain the ADR's registry-free decision.

In README:

- Use “identity slug” for the argument supplied to `capture` and `use`.
- Keep “identity” for saved auth entities and active-state messages.
- Update storage descriptions to `<identity slug>.json`.
- Document the breaking Rust rename from `IdentityName` to `IdentitySlug` and `Identity.name` to `Identity.slug` in the library feature section.

- [ ] **Step 3: Run final terminology and quality verification**

Run:

```bash
cargo fmt --all -- --check
cargo test
cargo test --features identity-details
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features
rg -n "IdentityName|InvalidIdentityName|SymlinkTargetHasInvalidIdentityName|identity\.name|display_name|Identity name|identity name|Auth slot" src README.md CONTEXT.md docs/adr
git diff --check
git status --short
```

Expected: every build and test command exits 0; all 15 all-feature tests pass; Clippy and rustdoc emit no warnings; `rg` returns no obsolete terms; diff check is empty. `git status` shows only this task's documentation plus the user's pre-existing `Cargo.toml` and `Cargo.lock` changes.

- [ ] **Step 4: Commit the documentation cleanup**

```bash
git add CONTEXT.md docs/adr/0001-use-identity-files-and-auth-json-symlink.md README.md
git commit -m "docs: define identity slugs"
```

---

## Completion check

After all four tasks, run:

```bash
git log -4 --oneline
git status --short
```

Expected commits, newest first:

```text
docs: define identity slugs
refactor!: rename identity detail name
refactor!: rename identity names to slugs
refactor: nest identity details module
```

The worktree must retain the user's uncommitted serde dependency changes for the next independent parsing plan.
