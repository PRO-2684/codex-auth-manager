# Identity Architecture Cleanup Design

## Scope

This is the first of three independently reviewable changes:

1. Rename the identity identifier consistently and deepen the identity module.
2. Replace dynamic JSON lookup with typed serde deserialization and make details failure policy explicit.
3. Display identity details from state-changing CLI commands.

This design covers only the first change. It deliberately preserves parsing and CLI behavior so the architectural change can be reviewed without behavioral noise.

## Goals

- Use one term, **Identity slug**, for the portable user-defined identifier stored as a managed auth filename stem.
- Reserve **Identity** for the saved Codex authentication entity.
- Make the Rust interface reflect that distinction everywhere.
- Concentrate identity details implementation beneath the identity module.
- Remove obsolete names rather than retain aliases or deprecations.
- Keep all current command behavior and output unchanged apart from Clap's argument label changing to `<SLUG>`.

## Non-goals

- Do not replace `serde_json::Value` parsing yet.
- Do not change malformed-details classification or CLI error suppression yet.
- Do not add details to `capture`, `use`, or `detach` yet.
- Do not add a metadata registry or change ADR-0001's identity-file storage design.
- Do not create a public storage abstraction.

## Domain language

`CONTEXT.md` will define:

**Identity slug**: The user-defined portable identifier of an **Identity**. CAM stores each managed auth file as `<identity slug>.json`.

The user-facing entity remains an **Identity**. Commands therefore remain `capture`, `use`, and `detach`, and messages continue to say “identity.” CLI positional arguments and Rust identifiers use `slug` when they carry only the identifier.

The existing accidental “Auth slot” reference in the Native auth file definition will be corrected to “Identity.”

ADR-0001 and README storage descriptions will use `<identity slug>` consistently. This refines ADR-0001's terminology without changing its decision.

## Public interface

The cleanup is intentionally breaking:

- `IdentityName` becomes `IdentitySlug`.
- `Identity.name` becomes `Identity.slug`.
- `IdentityDetails.display_name` becomes `IdentityDetails.name`.
- `AuthStatus::Managed { identity }` becomes `AuthStatus::Managed { slug }`.
- `AuthStatus::BrokenManaged { identity }` becomes `AuthStatus::BrokenManaged { slug }`.
- `Error::InvalidIdentityName { name }` becomes `Error::InvalidIdentitySlug { slug }`.
- Identity-not-found, identity-already-exists, and identity-broken error fields become `slug`.
- `UnknownAuthReason::SymlinkTargetHasInvalidIdentityName` becomes `SymlinkTargetHasInvalidIdentitySlug`.
- Manager method parameters and CLI command fields use `slug: IdentitySlug`.

No compatibility aliases, duplicate fields, deprecated variants, or conversion shims will remain.

## Module structure

The identity implementation will use the directory-root form:

```text
src/
└── identity/
    ├── mod.rs
    └── details.rs
```

`src/identity.rs` and `src/identity_details.rs` will be deleted after their implementations move.

`identity/mod.rs` owns:

- `Identity`
- `IdentitySlug`
- slug validation and formatting
- the conditional declaration and re-export of `IdentityDetails`

`identity/details.rs` owns:

- `IdentityDetails`
- `Display` for `IdentityDetails`
- auth-file reading
- JWT payload decoding
- claim sanitization
- `Identity::read_details`
- details parser tests

The existing broadly visible free reader will become crate-private type-owned behavior used by both `Identity::read_details` and `CodexAuthManager::read_active_auth_details`. The parser remains dynamically implemented during this change; typed serde structs belong to the second change.

The crate root re-exports `Identity`, `IdentitySlug`, and conditionally `IdentityDetails` from the identity module.

## Manager and CLI migration

All local callers move directly to the new interface:

- Listing, sorting, completion, capture, use, status, detach, path construction, and symlink parsing use `slug`.
- Error construction and pattern matching use `slug`.
- CLI positional fields are named `slug`, so generated help displays `<SLUG>`.
- User-facing prose continues to say “identity,” because the command acts on an Identity rather than on a storage mechanism.
- Existing enriched `list` and `status` formatting remains byte-for-byte unchanged.

## Storage mapping

The rename may expose repeated slug-to-managed-file transformations in the manager. Consolidation is permitted only when it deletes real duplication while keeping the implementation private. No new interface or adapter will be introduced for the single storage design.

## Testing

- Rename existing validation tests to use `IdentitySlug` and slug terminology.
- Update manager, status, error, completion, details, and CLI tests to the new fields and variants.
- Preserve exact assertions for current command output.
- Verify default, `identity-details`, and all-feature builds separately.
- Run formatter, Clippy with warnings denied, rustdoc with warnings denied, and `git diff --check`.
- Search committed source and documentation for obsolete interface terms: `IdentityName`, `identity.name`, `display_name`, `InvalidIdentityName`, and `SymlinkTargetHasInvalidIdentityName`.

## Breaking changes

All listed public renames are breaking. This is accepted because the project is still young and consistency is explicitly preferred over compatibility. The version receiving this change must document the new interface; no migration layer will be shipped.
