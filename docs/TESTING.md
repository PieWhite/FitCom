# Testing

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --lib
cargo test --locked -p fitcom-proto -p fitcom-store -p fitcom-net
cargo test --locked -p fitcom --test chat_sync --test codes_plakken --test file_deling --test stream_deling
cargo deny check -W unmaintained
```

Audio/video device tests may open capture devices, play audio, create windows, or
need OS permissions. Run deliberately on an interactive desktop. Unit tests do
not verify hardware, codecs, cross-platform media, firewall rules, or NAT reachability.

Use separate `--data-dir` directories, distinct ports outside production defaults,
and `port_mapping = false` for local instances. Never use real history or signing
keys for test data. The legacy `bind_address` field does not provide isolation.

Release verification includes clean-source inventory, credential scan, advisory
and license checks, signed-manifest checks, and anonymous feed/payload retrieval.
Test upgrading from the previous stable Windows build and verify identity, settings,
history, updater, and autostart survive. macOS runtime checks require a Mac.
