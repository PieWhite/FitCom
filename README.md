# FitCom

Native peer-to-peer chat, voice, screen sharing, and file sharing for small groups.
Windows 11 and macOS 14+ on Apple Silicon. Built with Rust and GPUI.

FitCom connects directly to peers you explicitly add. No FitCom account, central
chat server, signaling service, STUN server, or TURN relay is required. Each peer
stores its own identity, settings, and history.

**Experimental software for people you trust.** Connections authenticate peers and
media is encrypted, but relayed public-channel operations are not individually
signed. An admitted malicious peer can forge another author's messages and edits.
Do not use FitCom with hostile participants or where strong authorship guarantees
are required. See [security policy](SECURITY.md) and [audit](docs/SECURITY-AUDIT.md).

## Features

- Persistent chat, channels, direct messages, reactions, and reconnect synchronization.
- Opus voice, noise suppression, push-to-talk, and per-peer volume.
- Screen sharing, desktop audio, music, and optional remote control with local approval.
- Windows camera broadcasting; macOS can view Windows camera streams.
- File transfers, local clips/recordings, Windows tray and optional autostart.
- Signed Windows updates; YouTube previews and daily Wordle integration.

## Download and run

Use [FitCom releases](https://github.com/PieWhite/FitCom/releases).

**Windows:** extract into a folder owned by your user account. Keep `fitcom.exe`
and `fitcom-updater.exe` together. Launch `fitcom.exe`. An empty `data` folder beside
it enables portable mode; otherwise the OS application-data directory is used.
Existing installations retain their old data location, identity, settings, and chat.
Update-manifest signatures are separate from Windows Authenticode; publisher
signing and Apple notarization are not currently configured.

**macOS:** extract `FitCom.app`. Bundles currently use ad-hoc signing rather than
Apple Developer ID; macOS may require explicit approval and microphone/screen
recording permissions. Updates are manual. Camera broadcasting and autostart are
Windows-specific. Linux is not supported.

## Connect

1. Open **Settings → Network** and copy your invite code (`fitcom1:…`).
2. Exchange codes through a channel where you can verify the other person's identity.
3. Each participant adds the other's code under **Add peers**.

Codes include your public identity key, display name, and network addresses. Treat
them as personal information. Your code alone does not authorize a connection;
you must also admit the other peer's identity.

FitCom requests router mappings through UPnP, PCP, or NAT-PMP when enabled. Default
ports: `41650` (control), `41651` (media). Some CGNAT/firewall configurations prevent
direct connectivity. No hosted relay fallback exists. Permit the app only on
networks you trust; never disable your firewall to troubleshoot.

## Build

Requirements: Rust **1.97+**, CMake, and platform tools.

- Windows: MSVC toolchain, Visual Studio C++ Build Tools, Windows SDK (`fxc.exe`).
- macOS: Xcode command-line tools and `brew install cmake`.

```sh
cargo build --locked --release -p fitcom --bins
```

Windows binaries appear in `target/release/`. Distribute release builds with the
updater; debug builds can depend on build-machine shader sources. For macOS:

```sh
bash scripts/bundle-mac.sh
```

Bundles appear in `target/bundle/`.

## Development

```sh
cargo fmt --all -- --check
cargo test --locked --workspace --lib
cargo test --locked -p fitcom-proto -p fitcom-store -p fitcom-net
cargo deny check -W unmaintained
```

Hardware tests need real devices and an interactive desktop. See [testing](docs/TESTING.md)
and [contributing](CONTRIBUTING.md). Use separate `--data-dir` directories and
non-production ports for local instances.

## Updates

Windows checks a public HTTPS manifest about a minute after startup, every six hours, or when you
select **Check for updates**. Installed apps use no GitHub login, token, or API key.
The manifest is a release asset, not a GitHub REST API call. GitHub still sees
normal download metadata and can apply availability or abuse limits.

Updates require an embedded Ed25519 public key, signed version/size/BLAKE3 hash,
verified payload, and user confirmation. The updater checks the hash again before
replacement. Keep the private signing key outside the repository and archives.

A migration release is also offered through the previous release source so
existing clients can switch to FitCom. The original repository stays public until
its owner explicitly approves making it private after users migrate.
See [release procedure](docs/RELEASING.md).

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Privacy and local data](docs/PRIVACY.md)
- [Security policy](SECURITY.md) and [audit](docs/SECURITY-AUDIT.md)
- [Contributing](CONTRIBUTING.md), [testing](docs/TESTING.md), [releasing](docs/RELEASING.md)
- [Third-party notices](THIRD_PARTY_NOTICES.md)
- [Validation record](docs/VALIDATION.md)

## License

Project code: **GPL-3.0-only**, see [LICENSE](LICENSE). Dependencies retain their own
licenses. Bundled Wordle word-list licensing remains unverified and is not
represented as covered by the project's GPL grant. See [third-party notices](THIRD_PARTY_NOTICES.md).
