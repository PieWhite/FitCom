# Validation — 2026-10-29

Validation scope: local Windows x86-64 build and tests, plus dependency advisory,
license, source, and provenance review for the two supported platform graphs.
This is not a certification, a penetration test, or a two-machine media test.

- Library suite: 571 passed, 20 ignored. Ignored cases require hardware, an external
  service, or an explicitly requested measurement.
- Protocol/storage integration: 29 passed, including convergence and input validation.
- Network integration: 15 passed, plus the separate Windows timer test.
- App integration: 11 passed; one capture-dependent test remains ignored.
- Timer regression: 1 ms requested, 1.81 ms median, 7.38 ms maximum on isolated
  repeat. It failed at 22.4 ms median during heavy concurrent compilation; the
  threshold was retained and was not weakened to obtain a pass.
- cargo-audit: 1,043 locked packages checked against 1,386 current advisories;
  zero vulnerability advisories and zero yanked versions. Maintenance warnings
  and the unsupported GTK/Linux soundness finding remain documented in the audit.
- cargo-deny: all checks passed for Windows and Apple Silicon macOS with
  `-W unmaintained`; maintenance and duplicate-version warnings remain visible.
  No advisory IDs were ignored. Declared dependency licenses and allowed sources passed.
- Gitleaks: explicit source snapshot and fresh history from a single clean root scanned; no
  detected credentials. Pattern scanning does not prove the absence of all secrets
  or personal information. Local instructions, keys, runtime data, editor state,
  designs, historical working notes, and old Git objects were excluded.
- Update trust: the supplied external key was verified against the previous
  1.17.1 release tool's embedded public key. Protocol version remains 6.

Release builds also require native and Rust path remapping and a final scan of
actual executable bytes. Signed manifests and anonymous retrieval of published
payloads must pass before a release is considered validated. Release notes record
publication-specific results.

Manual Windows/macOS camera, capture, remote-control, NAT, and actual interactive
old-to-new installation testing were not performed in this environment. Read the
remaining limitations in SECURITY.md and SECURITY-AUDIT.md before use.
