# codex-auth-manager (cam)

[![GitHub License](https://img.shields.io/github/license/PRO-2684/codex-auth-manager?logo=opensourceinitiative)](https://github.com/PRO-2684/codex-auth-manager/blob/main/LICENSE)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/PRO-2684/codex-auth-manager/release.yml?logo=githubactions)](https://github.com/PRO-2684/codex-auth-manager/blob/main/.github/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/PRO-2684/codex-auth-manager?logo=githubactions)](https://github.com/PRO-2684/codex-auth-manager/releases)
[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/PRO-2684/codex-auth-manager/total?logo=github)](https://github.com/PRO-2684/codex-auth-manager/releases)
[![Crates.io Version](https://img.shields.io/crates/v/codex-auth-manager?logo=rust)](https://crates.io/crates/codex-auth-manager)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/codex-auth-manager?logo=rust)](https://crates.io/crates/codex-auth-manager)
[![docs.rs](https://img.shields.io/docsrs/codex-auth-manager?logo=rust)](https://docs.rs/codex-auth-manager)

A deadly simple Codex auth manager.

## 📥 Installation

### Using [`binstall`](https://github.com/cargo-bins/cargo-binstall)

```shell
cargo binstall codex-auth-manager
```

### Downloading from Releases

Navigate to the [Releases page](https://github.com/PRO-2684/codex-auth-manager/releases) and download respective binary for your platform. Make sure to give it execute permissions.

### Compiling from Source

```shell
cargo install codex-auth-manager --features=cli
```

## 💡 Examples

```shell
cam capture work
cam list
cam use personal
cam detach
```

## 📖 Usage

`cam` manages named Codex auth identities. Each identity is stored under `$CODEX_HOME/codex-auth-manager/`, and CAM switches which identity Codex sees at `$CODEX_HOME/auth.json`.

Planned v1 CLI:

- `cam` / `cam status` — show the current auth state
- `cam list` — list saved identities
- `cam capture <identity> [--force]` — save the current native Codex auth file as an identity and make it active
- `cam use <identity> [--force]` — make an existing identity active
- `cam detach [--force]` — stop using the active CAM-managed identity

## 🎉 Credits

TODO
