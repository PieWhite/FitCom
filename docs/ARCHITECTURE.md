# FitCom architecture

FitCom is a native Rust application for small, mutually trusted groups. Every
instance initiates and accepts peer connections; no designated host exists.
Offline peers are normal and reconnect synchronization must remain available.

| Crate | Responsibility |
| --- | --- |
| `fitcom-proto` | Wire types, peer IDs, invitation codes, operation encoding |
| `fitcom-net` | QUIC connections, identity proofs, framing, routing, encrypted media |
| `fitcom-store` | SQLite operation log and derived timelines |
| `fitcom-audio` | Opus, capture, mixing, noise suppression, process loopback |
| `fitcom-video` | OS capture, hardware codecs, stream display and remote control |
| `fitcom` | Application engine, configuration, UI, files, update client/tools |

The engine publishes snapshots and accepts typed commands. GPUI draws native views
without a webview or JavaScript runtime. Video viewers have separate platform
rendering paths. UI code must not block peer synchronization or audio processing.

## Trust and synchronization

Invite codes pin identity keys. Ed25519 identity proofs are bound to TLS exporters;
media keys derive from the authenticated session. The store applies immutable,
idempotent operations and synchronizes missing history after reconnecting.

Public operations can be relayed by another peer. They do not yet contain an
independent author signature, so authenticated connection identity does not prove
public-message authorship. Direct-message operations have stricter sender checks.
See [security audit](SECURITY-AUDIT.md) for the remaining trust limitation.

## Networking and privacy

Router port mapping uses UPnP/PCP/NAT-PMP when enabled. Peer addresses can be learned
through authenticated connections. No hosted signaling, STUN, or TURN relay is
provided. Port mapping does not guarantee reachability through CGNAT.

External HTTPS requests also support release checks, YouTube previews, and daily
Wordle puzzles. These are documented in [privacy](PRIVACY.md); the application is
not completely disconnected from third-party services.

## Platforms and compatibility

Windows uses DirectX/Media Foundation and Windows audio APIs. macOS uses Metal,
ScreenCaptureKit, VideoToolbox, and native audio APIs. macOS updates are manual;
camera broadcasting and autostart currently require Windows.

Preserve existing wire tags and defaults when adding fields. Breaking security
changes require a documented coordinated migration and tests with old data.
The legacy application-data directory, Windows mutex namespace, and macOS bundle
identifier remain compatibility identifiers where needed; displayed branding is FitCom.

## Releases

`fitcom-release` signs a size/hash/version manifest using an external Ed25519 key.
Windows clients download public GitHub release assets, validate the manifest and
payload, and invoke `fitcom-updater` only after user approval. The updater rechecks
the hash before replacing the executable. See [releasing](RELEASING.md).
