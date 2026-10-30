# Security review — 2026-10-29

This review covers source publication, peer authentication, transport encryption,
file handling, remote control, image previews, release verification and the locked
Rust dependencies. It is a focused source review and dependency scan, not a security
certification or a penetration test. Windows and macOS are the supported targets.

## Findings that remain

| Severity | Finding | Status and recommendation |
| --- | --- | --- |
| High: integrity | Public-channel operations have no author signature. | An authenticated, invited peer can claim another author when relaying public operations. This permits forged messages, profile changes, edits and deletions. Direct messages require the authenticated sender to match the author. Public relaying is intentional, but connection authentication alone does not prove the original author. Use only with trusted peers. A complete repair requires signed operations, pinned author keys, a migration for historical unsigned data, and protocol compatibility tests. |
| Medium: dependency soundness, unsupported target | `glib` 0.18.5 remains in the cross-platform lockfile. | [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) is fixed in 0.20.0+. This comes through the GTK tray dependencies on Linux/BSD, rather than the Windows/macOS tray backend. Do not infer that the unsupported Linux build is security-reviewed. Upgrade the owning GTK/tray dependency before adding Linux support. |
| Maintenance risk | Ten dependencies are reported as unmaintained. | Listed below. These advisories do not by themselves demonstrate a reachable exploit. Track upstream replacements instead of silently suppressing them. The native Opus binding deserves particular attention because it processes peer-supplied audio. |
| Publication provenance | The bundled Wordle guess list has no independently verified redistribution license. | The source identifies it as copied from a published game client. The maintainer chose to retain it. Ownership and redistribution permission were not independently established by this review; the project's license must not be taken as proof of third-party rights. |
| Notice provenance | Nine crates declare licenses but do not supply a standalone upstream notice. | `mac` 0.1.1, `realfft` 3.5.0, `seahash` 4.1.0, `simd_helpers` 0.1.0, `block` 0.1.6, `leak` 0.1.2, `leaky-cow` 0.1.1, `malloc_buf` 0.0.6 and `objc_exception` 0.1.2 declare MIT and/or Apache-2.0 licensing in their published packages. Their declarations, upstream attribution and canonical license text are retained in the combined notice bundle; missing package-specific copyright wording has not been invented. The authoritative `seahash` GitLab host returned HTTP 403. This is a notice-provenance limitation, not a newly discovered vulnerability or an assertion that those packages are unlicensed. |

The author-authentication finding is enforced by the current implementation in
`crates/store/src/lib.rs::keur_op`: a differing author is rejected only when the
channel is not public. `crates/store/tests/invoercontrole.rs` explicitly preserves
public relaying. The timeline's ownership checks compare the claimed author with
the target author, so they do not repair this gap.

An invited peer is trusted with public-channel content. This is not an appropriate
security model for adversarial communities or mutually untrusted users.

Follow-up, 2026-10-30: this finding is still **open in 1.18.1 / protocol 6**.
The [remediation task](SECURITY-REMEDIATION.md) records implementation entry points,
key-binding requirements, unsigned-history migration hazards and acceptance tests,
including a Claude handoff prompt. No operation-signature repair has shipped.

## Protections verified in the current source

- **Peer identity:** Ed25519 proofs bind the identity to a role-specific TLS exporter.
  Pinned public keys are checked before a normal `HelloAck` is sent. The custom TLS
  certificate verifier is therefore not the application identity check. Initial
  trust still depends on exchanging the correct invitation code or on the explicitly
  configured first-contact trust path.
- **Media:** directional AES-256-GCM keys derive from the authenticated QUIC session.
  Packet counters and a replay window reject modified or replayed packets. Roaming
  learns addresses only after successful authentication. An on-path attacker can
  still delay/drop traffic or race a fresh packet to influence its return address;
  this does not reveal the media key.
- **Public listener:** QUIC retry, per-IP connection limits, concurrent connection
  limits, handshake deadlines, incremental bounded framing, and bounded log output
  reduce pre-authentication resource attacks. These do not prove immunity to a
  distributed denial of service.
- **File transfer:** incoming transfers must correspond to an outstanding request
  from the correct peer; advertised lengths and content hashes are checked. File
  names are sanitized and image cache names derive from content hashes.
- **Private-key files, hardened during this review:** identity and release-key
  writes set Unix permissions to `0600`; loading an existing identity also restricts
  its permissions. Release key generation uses exclusive creation, so an existing
  key cannot be overwritten through a check-then-create race. Windows inherits
  the containing directory's ACL: portable data and signing keys must still be
  stored in a private, user-owned directory.
- **Remote control:** disabled by default; per-session local approval is required.
  Input is accepted only from the approved viewer and stops on revocation or loss of
  the relevant session. Clipboard sharing is limited to that approved session.
  Approval gives substantial control over the shared desktop, including programs
  visible on that monitor; it is not a restricted application sandbox.
- **Updates:** the app downloads public HTTPS release assets without a GitHub login
  or personal API token. Ed25519 authenticates the version, BLAKE3 hash and size;
  the binary is size-bounded and checked against that signed hash. The updater
  rehashes after the main process exits and before replacement. This closes the
  historical long waiting-window issue, but is not a defense against arbitrary
  local code already running as the same user. Legacy updater compatibility still
  permits invocation without a hash.
- **Image previews, repaired during this review:** actual content type is checked,
  encoded input is limited to 16 MiB, and decoded dimensions are limited to 8192 per
  side and 16,777,216 pixels. GIFs use ordinary file cards because the GPUI loader
  retains every animation frame without an aggregate allocation limit. A GIF
  renamed to `.png` is also rejected for inline rendering. PNG, JPEG and BMP retain
  static inline previews. Tests cover renamed animations and encoded/decoded
  memory limits.

These source checks confirm that the historical identity-spoofing, unauthenticated
media, unsolicited-file, file-path traversal and unsigned-update designs have been
replaced. Old reports describing those designs as current are obsolete. Public
operation author spoofing is a separate unresolved issue.

## Source-publication scan

The explicit clean export was scanned with **Gitleaks 8.30.1**, using the official
Windows release binary after verifying its published SHA-256 digest. The directory
scan covered approximately **3.43 MB** and reported **zero detected secrets**, with
exit code 0. Findings and logs were redacted; inline allow-comments were ignored.

The export was also checked for local assistant instructions, private key formats,
runtime databases, logs, recordings and debug symbols. None of those local-only
files appeared. Matches for personal-name/path patterns were reviewed: retained
matches were dictionary words, fictional test data or generic path examples.
Stable legacy storage, mutex and application identifiers remain where needed for
existing installations; they are compatibility strings rather than visible product
branding.

This is evidence about the reviewed export, not a guarantee that every possible
credential or personal identifier can be recognized by automated rules. Re-run the
scan after changing the release source snapshot or adding generated assets.

## Dependency scan

The baseline lockfile had **11 vulnerability advisories**, plus soundness and
maintenance warnings. Package presence in a lockfile is not proof that every
reported function or optional feature is reachable.

| Package | Baseline | Required/current repair | Official advisory |
| --- | --- | --- | --- |
| `quinn-proto` | 0.11.16 | 0.11.19 | [0338](https://rustsec.org/advisories/RUSTSEC-2026-0338.html), [0339](https://rustsec.org/advisories/RUSTSEC-2026-0339.html), [0340](https://rustsec.org/advisories/RUSTSEC-2026-0340.html), [0341](https://rustsec.org/advisories/RUSTSEC-2026-0341.html), [0342](https://rustsec.org/advisories/RUSTSEC-2026-0342.html) |
| `rustls` | 0.23.42 | 0.23.46 | [0285](https://rustsec.org/advisories/RUSTSEC-2026-0285.html), [0346](https://rustsec.org/advisories/RUSTSEC-2026-0346.html), [0348](https://rustsec.org/advisories/RUSTSEC-2026-0348.html), [0349](https://rustsec.org/advisories/RUSTSEC-2026-0349.html) |
| `rustls-webpki` | 0.103.13 | 0.103.16 | [0382](https://rustsec.org/advisories/RUSTSEC-2026-0382.html) |
| `webpki-roots` | 1.0.9 | 1.0.10 | [0381](https://rustsec.org/advisories/RUSTSEC-2026-0381.html) |
| `tokio` | 1.53.1 | 1.53.3 | [0386, Unix `AsyncFd` soundness](https://rustsec.org/advisories/RUSTSEC-2026-0386.html) |
| `wasapi` | 0.23.0 | 0.25.0 | [0332, `WaveFormat::parse` soundness](https://rustsec.org/advisories/RUSTSEC-2026-0332.html) |
| `chacha20` | 0.10.1 (yanked) | 0.10.2 | [Upstream fix for undefined behavior in the SSE2 RNG/legacy backend](https://github.com/RustCrypto/stream-ciphers/pull/580) |

The final updated-lockfile scan used the official **cargo-audit 0.22.2** Windows
binary, verified against the SHA-256 digest published with its GitHub release. It
checked **1,043 dependencies against 1,386 advisories**, using the official RustSec
database updated on 2026-10-29, and completed with **zero vulnerability advisories**
and exit code 0. Ten maintenance warnings and the GTK `glib` soundness warning remain.
No advisory exceptions were supplied.

The final scan also included registry yanked-version checks. It found **zero yanked
dependencies** after replacing `chacha20` 0.10.1 with 0.10.2. Upstream withdrew the
old version because its SSE2 backend could execute an SSE4.1 intrinsic without the
required CPU guarantee. No operating-system or architecture filter was applied.
A clean vulnerability count does not remove the following warnings or application
findings.

| Unmaintained package | Locked version | Advisory |
| --- | --- | --- |
| `audiopus_sys` | 0.2.2 | [RUSTSEC-2026-0150](https://rustsec.org/advisories/RUSTSEC-2026-0150.html) |
| `instant` | 0.1.13 | [RUSTSEC-2024-0384](https://rustsec.org/advisories/RUSTSEC-2024-0384.html) |
| `paste` | 1.0.15 | [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) |
| `proc-macro-error` | 1.0.4 | [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html) |
| `rustls-pemfile` | 2.2.0 | [RUSTSEC-2025-0134](https://rustsec.org/advisories/RUSTSEC-2025-0134.html) |
| `rustybuzz` | 0.20.1 | [RUSTSEC-2026-0206](https://rustsec.org/advisories/RUSTSEC-2026-0206.html) |
| `ttf-parser` | 0.25.1 | [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html) |
| `zbus_names` | 4.3.4 | [RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html) |
| `zvariant` | 5.15.0 | [RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html) |
| `zvariant_derive` | 5.15.0 | [RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html) |

## License policy and validation

Official **cargo-deny 0.20.2**, with its release archive SHA-256 verified, checked
licenses and source registries separately against the actual locked Windows and
Apple Silicon macOS dependency graphs. Both checks passed with zero errors and
zero warnings: 572 accepted license decisions on Windows and 605 on macOS. The
policy explicitly allows `CDLA-Permissive-2.0`, the license on the updated
`webpki-roots` data package. These checks validate declared licenses and policy;
they do not independently prove ownership of each contribution or asset.

The full advisory, dependency-ban, license and source checks also passed on both
supported target graphs using `cargo deny check -W unmaintained`. Only the
unmaintained-package diagnostic was explicitly treated as a warning; known
vulnerabilities, soundness failures, forbidden licenses and unknown sources remain
errors. No advisory IDs were ignored. Each platform reported five maintenance
warnings; duplicate dependency versions remain visible warnings (36 on Windows,
28 on macOS). Internal path dependencies now have explicit version constraints.

The combined [notice bundle](../THIRD_PARTY_LICENSES.txt) accounts for 735 distinct
package/version pairs across the locked Windows and macOS graphs. It supplements
67 crates whose packaged source omitted their monorepo's root notices, using
files from the exact source commits recorded in published Cargo VCS metadata.
It also preserves published source-header notices for `pathfinder_geometry`,
native Opus license/copyright/patent notices separately from the Rust binding's
license, and all three bundled font license files. The nine remaining
standalone-notice gaps are disclosed above.

The completed workspace library test run reported **571 passed, 20 ignored**.
Ignored hardware/network tests are not counted as coverage, and subsequent edits
or a different final release build still require the release validation procedure.

## Publication and release limits

- A fresh repository must start from an explicit clean-file snapshot, not a branch
  retaining the old Git objects. Removing a file from the latest commit does not
  remove its history. Keep assistant instructions, editor sessions, runtime data,
  signing keys, build output and private working notes outside that snapshot.
- Public history and release assets can already have been copied. Changing
  repository visibility does not revoke a leaked credential or retract existing
  copies. Rotate any credential actually exposed; do not rotate the update-signing
  key casually because installed apps pin its public key.
- Existing installed versions continue to use their compiled-in update URL. Keep
  the old public feed available until users have installed the migration release.
  A private repository cannot serve those anonymous clients.
- Release stripping is not sufficient proof of privacy: panic strings, native
  source paths, embedded resource metadata and recordings can survive. Remap build
  paths and inspect the final distributed binaries and archives before publishing.
- Dependency source licenses and notices must accompany binary distribution where
  required. A project license does not replace dependency licenses or establish
  rights to separately sourced assets.
- This review did not audit the internals of native Opus, SQLite, the operating
  system media decoders or every GPUI dependency, and did not perform a two-machine
  Windows/macOS adversarial network test. Test results and final release validation
  must be recorded separately; added tests are not assumed to have passed.
