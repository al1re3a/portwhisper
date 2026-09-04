# PortWhisper

[![CI](https://github.com/al1re3a/portwhisper/actions/workflows/ci.yml/badge.svg)](https://github.com/al1re3a/portwhisper/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

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
