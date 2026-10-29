# Contributing to FitCom

Describe the problem and expected behavior before proposing large changes. Keep
changes focused. Include regression tests for security and protocol fixes. Never
attach private logs, databases, identities, or invite codes.

Use Rust 1.97+ and platform prerequisites from the README. Run `cargo fmt --all`,
library tests, and protocol/storage/network integration tests. Audio/video changes
also need real hardware testing. See [testing](docs/TESTING.md).

Keep UI code separate from protocol, storage, networking, audio, and video crates.
Every peer must keep working when others disconnect. Protocol changes must preserve
compatibility or document a coordinated migration. Bound untrusted sizes, counts,
paths, and allocations before processing peer input.

Commit `Cargo.lock`. Review dependency advisories and licenses. Never commit signing
keys, tokens, runtime data, editor sessions, or local assistant instructions.
Run `scripts/check-public-source.ps1` before publishing.

Contributions to project code are under GPL-3.0-only. Include original licenses for
third-party work; public availability alone does not grant redistribution rights.
