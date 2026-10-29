# Releasing FitCom

Release code and binaries come from `PieWhite/FitCom`. Never publish a branch or
tag descended from the original repository's personal commit history. The public
repository begins with a clean snapshot and a public-safe Git author identity.

## Before a release

1. Review [security audit](SECURITY-AUDIT.md) and unresolved third-party provenance.
2. Set the workspace version. Build/test with the committed `Cargo.lock`.
3. Run formatting, tests, `cargo deny check -W unmaintained`, and a credential scan. Treat scanner
   warnings separately from confirmed reachable vulnerabilities; document any exceptions.
4. Run `scripts/check-public-source.ps1`. Assistant files, local notes, runtime data,
   keys, editor state, recordings, build output, and old history must stay untracked.
5. Build with `scripts/build-release.ps1` on Windows. It remaps local build paths
   and rejects binaries containing the builder's home path. Never ship PDB files.

## Sign and stage Windows artifacts

Keep the existing signing key outside the repository and any bundle. Back it up
privately. The public key in `crates/app/src/release.rs` is intentionally public.
Do not regenerate the key: installed applications would reject the new signatures.

```powershell
.\target\release\fitcom-release.exe sign --key <external-key-path> `
  --exe target\release\fitcom.exe --version <workspace-version> `
  --url https://github.com/PieWhite/FitCom/releases/download/v<workspace-version>/fitcom.exe `
  --out .private\release\latest.json
.\target\release\fitcom-release.exe verify --manifest .private\release\latest.json
```

Publish a tag from the tested source commit. Stage a draft release containing
`fitcom.exe`, `fitcom-updater.exe`, `latest.json`, a user ZIP with license/notices,
and corresponding source at that exact tag. Do not publish the signing tool or key.
Include applicable dependency notices with binaries; the project GPL does not
replace third-party license obligations.

After publishing, run `fitcom-release check`, then anonymously download the
manifest and actual executable and verify its signature/hash/size. A successful
HEAD request alone does not prove a working update. Test an actual previous-version
upgrade on an interactive Windows machine.

## One-time migration from the previous repository

Older installed clients accept download URLs only inside their original repository.
Publish the migration build on **both** repositories under the same new version.
The binaries and signed version/hash/size are identical, but each manifest must
name a payload URL within the repository its clients already trust. The URL itself
is constrained by the client rather than included in the signed message.

The migration binary has the new FitCom manifest URL compiled in. After updating,
later checks use FitCom automatically. Keep `fitcom-updater.exe` beside the main
executable; changing the main binary does not automatically replace an old updater.
The complete user ZIP supplies both.

Both repositories stay public until the human owner confirms everyone has migrated
and explicitly approves changing the old repository to private. Do not automate
that visibility change. Existing history/assets may already have been copied;
private visibility cannot retract copies or revoke an exposed credential.

## macOS

`bash scripts/bundle-mac.sh` creates a FitCom bundle with the existing bundle
identifier retained for compatibility. The workflow builds a downloadable artifact
and can attach it to an existing release. Signing is ad-hoc, not Apple notarization.
Validate capture permissions and cross-platform connectivity on real hardware.
