<!-- readme-refresh:start -->
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/readme-banner.png">
    <source media="(prefers-color-scheme: light)" srcset="assets/readme-banner.png">
    <img alt="PortWhisper project banner" src="assets/readme-banner.png" width="100%">
  </picture>
</p>

<h1 align="center">🔌 PortWhisper</h1>

<p align="center"><strong>Inspect listening ports with clear exposure labels and portable output.</strong></p>

<p align="center">
  <a href="https://github.com/al1re3a/portwhisper/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/al1re3a/portwhisper/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://www.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/Rust-2021%20edition-B7410E?logo=rust&logoColor=white"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-fbbf24.svg"></a>
  <a href="https://github.com/al1re3a/portwhisper/stargazers"><img alt="GitHub stars" src="https://img.shields.io/github/stars/al1re3a/portwhisper?style=flat&color=8b5cf6"></a>
  <a href="https://github.com/al1re3a/portwhisper/issues"><img alt="Open issues" src="https://img.shields.io/github/issues/al1re3a/portwhisper?style=flat&color=06b6d4"></a>
</p>

<p align="center">
  <a href="https://github.com/al1re3a/portwhisper"><img alt="Source" src="https://img.shields.io/badge/Source-open-111827?style=for-the-badge&logo=github&logoColor=white"></a>
  <a href="#install-and-develop"><img alt="Quick Start" src="https://img.shields.io/badge/Quick_Start-open-0f766e?style=for-the-badge&logo=gnubash&logoColor=white"></a>
  <a href="CONTRIBUTING.md"><img alt="Contribute" src="https://img.shields.io/badge/Contribute-open-7c3aed?style=for-the-badge&logo=github&logoColor=white"></a>
  <a href="SECURITY.md"><img alt="Security" src="https://img.shields.io/badge/Security-open-b91c1c?style=for-the-badge&logo=securityscorecard&logoColor=white"></a>
</p>

<p align="center">
  <img src="https://skillicons.dev/icons?i=rust,githubactions" alt="Rust and GitHub Actions" height="42">
</p>

> [!NOTE]
> Exposure labels describe bind addresses. Firewall policy and network reachability still need separate verification.

## 📑 Contents

- [At a glance](#-at-a-glance)
- [Exposure labels](#exposure-labels)
- [Install and develop](#install-and-develop)

---

## 🔎 At a glance

| | |
|---|---|
| **Purpose** | Cross-platform listening-port inspector with exposure labels, JSON and Mermaid output. Rust, no dependencies. |
| **Input** | OS socket tables |
| **Output** | Table, JSON, or Mermaid |
| **Runtime** | Rust 2021 edition |
| **CI** | ✅ Linux · macOS · Windows |
| **Status** | ✅ Maintained |

<details>
<summary><strong>🧭 How it works</strong></summary>

```mermaid
flowchart LR
    A["OS socket tables"] --> B["Classify listeners"]
    B --> C["Table, JSON, or Mermaid"]
```

</details>

<details>
<summary><strong>📁 Repository layout</strong></summary>

```text
portwhisper/
├── .github/
├── src/
├── examples/
├── Cargo.toml
└── README.md
```

</details>

<details>
<summary><strong>🤝 Contributors</strong></summary>

<br>
<a href="https://github.com/al1re3a/portwhisper/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=al1re3a/portwhisper" alt="Contributors">
</a>

</details>
<!-- readme-refresh:end -->

Explain which TCP ports are listening, which process owns them when the operating system reports it, and whether they are bound locally or exposed to a network.

```console
portwhisper
portwhisper --format json
portwhisper --format mermaid > ports.mmd
```

PortWhisper uses built-in operating-system tools: `netstat` on Windows, `ss` on Linux, and `lsof` on macOS. No packets are sent and no ports are opened. Process details may require the same privileges as the owning process.

It can also parse captured output, which is useful for support tickets and CI fixtures:

```console
portwhisper --input examples/windows-netstat.txt --source windows
```

## Exposure labels

- `local`: loopback only, normally reachable just from this host.
- `network`: bound to one non-loopback address.
- `all-interfaces`: wildcard binding; review firewall and application authentication.

An exposure label is not proof that a port is reachable through a firewall.

## Install and develop

```console
cargo install --path .
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

MIT licensed. Contributions are welcome.
