# codex-auth-manager

[![GitHub License](https://img.shields.io/github/license/PRO-2684/codex-auth-manager?logo=opensourceinitiative)](https://github.com/PRO-2684/codex-auth-manager/blob/main/LICENSE)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/PRO-2684/codex-auth-manager/release.yml?logo=githubactions)](https://github.com/PRO-2684/codex-auth-manager/blob/main/.github/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/PRO-2684/codex-auth-manager?logo=githubactions)](https://github.com/PRO-2684/codex-auth-manager/releases)
[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/PRO-2684/codex-auth-manager/total?logo=github)](https://github.com/PRO-2684/codex-auth-manager/releases)
[![Crates.io Version](https://img.shields.io/crates/v/codex-auth-manager?logo=rust)](https://crates.io/crates/codex-auth-manager)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/codex-auth-manager?logo=rust)](https://crates.io/crates/codex-auth-manager)
[![docs.rs](https://img.shields.io/docsrs/codex-auth-manager?logo=rust)](https://docs.rs/codex-auth-manager)

Codex auth manager

## ⚙️ Automatic Releases Setup

1. [Create a new GitHub repository](https://github.com/new) with the name `codex-auth-manager` and push this generated project to it.
2. Enable Actions for the repository, and grant "Read and write permissions" to the workflow [here](https://github.com/PRO-2684/codex-auth-manager/settings/actions).
3. [Generate an API token on crates.io](https://crates.io/settings/tokens/new), with the following setup:

    - `Name`: `codex-auth-manager`
    - `Expiration`: `No expiration`
    - `Scopes`: `publish-new`, `publish-update`
    - `Crates`: `codex-auth-manager`

4. [Add a repository secret](https://github.com/PRO-2684/codex-auth-manager/settings/secrets/actions/new) named `CARGO_TOKEN` with the generated token as its value.
5. Consider removing this section and updating this README with your own project information.

[Trusted Publishing](https://crates.io/docs/trusted-publishing) is a recent feature added to crates.io. To utilize it, first make sure you've already successfully published the crate to crates.io. Then, follow these steps:

1. [Add a new trusted publisher](https://crates.io/crates/codex-auth-manager/settings/new-trusted-publisher) to your crate.
    - Set "Workflow filename" to `release.yml`.
    - Keep other fields intact.
    - Click "Add".
2. Modify [`release.yml`](.github/workflows/release.yml).
    1. Comment out or remove the `publish-release` job.
    2. Un-comment the `trusted-publishing` job.
3. Remove the `CARGO_TOKEN` [repository secret](https://github.com/PRO-2684/codex-auth-manager/settings/secrets/actions).
4. Revoke the API token on [crates.io](https://crates.io/settings/tokens).

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

TODO

## 📖 Usage

TODO

## 🎉 Credits

TODO
