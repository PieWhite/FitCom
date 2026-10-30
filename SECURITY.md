# Security policy

FitCom is experimental software for small groups of mutually trusted peers.
Only the latest release is supported. Install updates promptly.

## Reporting vulnerabilities

Use [private vulnerability reporting](https://github.com/PieWhite/FitCom/security/advisories/new)
when available. Never post real invite codes, identity keys, signing keys, chat
databases, or unredacted logs in public issues. If private reporting is unavailable,
request a private contact without disclosing exploit details. No response-time
commitment is guaranteed.

## Security boundaries and known limitations

- Connection identities use pinned keys; verify invite codes out of band.
- Public-channel operations can be relayed and are not individually signed.
  An admitted malicious peer can forge another author's messages, edits, or deletions.
- Local data, identity material, received files, and logs are not encrypted at rest.
- Router mappings can expose listening ports to the internet. The historical
  `bind_address` field does not constrain actual listening interfaces. Do not use
  it for isolation or disable your firewall to troubleshoot.
- Received files are not malware-scanned. Approve remote control only for trusted peers.
- Resource limits reduce denial-of-service risks but do not eliminate them.
- Updates trust an embedded Ed25519 public key. A leaked private key requires a
  coordinated manual replacement. A GitHub token alone cannot sign releases.

See [current audit](docs/SECURITY-AUDIT.md) for patches and remaining findings.
The public-operation author-spoofing issue remains **open**; its
[remediation task](docs/SECURITY-REMEDIATION.md) specifies the required signing,
trust, migration and regression-test work. A documented plan is not a patch.
Code review and dependency scans are not a formal penetration test or a guarantee
that the application has no vulnerabilities.
