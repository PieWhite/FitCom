# Public-operation author authentication: open remediation task

Status: **OPEN / NOT FIXED** as of 2026-10-30. Released FitCom 1.18.1 uses
protocol 6 and store schema 2. This document specifies work; it is not evidence
that operation signatures or a migration have been implemented.

## Problem and current evidence

An admitted, authenticated peer can invent a public operation with another peer's
`author`. This affects messages, profile changes, edits and deletions. Forged
operations can also occupy an author's `(author, channel, seq)` key before the
genuine operation arrives. Transport authentication proves the immediate sender,
not the original author of relayed content.

Relevant code:

- `crates/proto/src/op.rs`: unsigned `Op`, payload encoding and version vectors.
- `crates/proto/src/ids.rs`: `PeerId` is a random UUID, not a public-key digest.
- `crates/proto/src/lib.rs`: `PROTOCOL_VERSION = 6`.
- `crates/store/src/lib.rs`: `keur_op` permits a differing author on public channels;
  insertion, duplicate handling, contiguous frontiers and schema migration live here.
- `crates/store/src/timeline.rs`: derives displayed state, including edit/delete
  authorization, from claimed authors and target IDs.
- `crates/store/tests/invoercontrole.rs`: the B-06 public-relaying test deliberately
  accepts a third party's unsigned claim; B-07 covers sequence-key collisions.
- `crates/app/src/chat.rs`: both broadcasts and sync responses enter `neem_over`;
  accepted public operations are rebroadcast.
- `crates/app/src/main.rs`, `config.rs`, `engine.rs` and `codes.rs`: local identity,
  stored peer pins, first-contact learning and invite/config updates.
- `crates/net/src/mesh.rs`: authenticated connections and `LearnedIdentity` events.

On 2026-10-30 this focused baseline check passed:

```text
cargo test --locked -p fitcom-store --test invoercontrole b06_een_publieke_op_mag_wel_via_een_derde_peer_binnenkomen -- --exact
```

It confirms unsigned third-party public operations are still accepted. It is a
baseline behavior test, not evidence that this vulnerability has been repaired.
See [the audit](SECURITY-AUDIT.md) and [security policy](../SECURITY.md).

## Required security outcome

Keep legitimate public forwarding, offline synchronization and idempotent
operations. A receiver must independently authenticate the original author before
an operation changes trusted state, occupies a trusted sequence key, advances a
trusted frontier, increments unread state or is forwarded. Do not solve this by
requiring `sender == author` for public channels: that breaks partial-connectivity
delivery. Direct messages must retain their routing and recipient restrictions.

Use the existing per-installation Ed25519 identity key for operation signing.
The release-signing key is a separate trust domain and must not be used, opened or
uploaded for this work.

Define and document a deterministic, versioned signing representation with domain
separation. Bind every semantic operation field: author, channel and its target,
sequence, Lamport value, wall-clock value, kind tag and the exact opaque payload
bytes. Bind any new epoch, migration or signing-version fields as well. Specify
lengths and integer encoding; do not depend on map iteration order or decoding and
re-encoding known payload variants. Authentic unknown operation kinds must remain
forwardable without interpretation. Reject malformed/unsupported signature forms
and size-bound all new fields and verification work.

Current `Channel` decoding normalizes unknown tags. Define canonical channel
semantics explicitly: reject unsupported signed channel/envelope formats or
preserve their raw authenticated extension fields. Do not normalize away a future
security-relevant field and then report its altered operation as authenticated.

The author's public key must match an independently trusted binding for that
`PeerId`. An attacker-controlled envelope containing both a key and a valid
signature is not sufficient: UUIDs are not self-authenticating. Neither an
authenticated relay nor its own pinned key may vouch for arbitrary third-party
UUID/key bindings. Unknown authors require an explicit safe trust flow; no silent
trust-on-first-use from relayed operations. A changed key must not silently replace
an existing pin. Configuration load, first-contact learning, invite addition,
removal and restart must all keep verification policy consistent.
Define revocation separately from historical verification: retaining a key to
render previously accepted history must not authorize new operations after its
trust is removed. Replacing or re-adding a pin needs explicit key-change semantics.

Persist the original authenticated envelope and return it unchanged during sync.
Verification must cover every network ingestion path, including batches, and
there must be no production fallback to an unchecked store import API. Preserve
existing resource limits, DM confidentiality and collision detection.

## Migration is part of the fix

Historical unsigned records cannot prove original authorship. They may already
contain fabricated operations. Preserve user data and identity, but never silently
promote received historical data to authenticated content or sign another author's
records. The existing database stores no reliable creation provenance: even rows
claiming our own UUID may be old incoming forgeries. Do not automatically re-sign
those rows either. A later explicit endorsement would be a new action, not proof
of original authorship.

The independent review recommends evaluating protocol 7 and schema 3 with a new
signed-operation namespace whose sequences start at 1, retaining all schema-2
records in an explicitly unverified legacy archive. This needs versioned operation
references, SQLite primary keys and version-vector keys to prevent old/new ID
collisions, plus an intentional read-only archive projection/UI. It is
a candidate design, not an approved migration or an implemented feature; check
its effects on replies, files, profiles, DMs and recovery before choosing it.

Before implementation, choose and document a coherent policy for:

1. Displaying locally retained legacy history and distinguishing its unverified
   status. Legacy edits, deletes and profile state must not override authenticated
   post-migration state.
2. Syncing legacy history, recovering a missing database and starting a fresh
   installation while an original author is offline. State clearly what cannot be
   authenticated or recovered without that author/trust material.
3. Sequence numbers and version vectors. Dropping unsigned records while retaining
   a contiguous frontier can skip data permanently; refusing an unsigned prefix
   can also leave an otherwise valid signed tail permanently outside the window.
   Do not solve gaps by trusting frontiers asserted by a relay. Consider an
   explicitly authenticated epoch/checkpoint or a separate legacy archive, with
   consistent operation references and DM semantics.
4. Existing replies, reactions, file references, channel IDs and edit/delete
   targets. Do not silently reset IDs or make old references point to new content.
5. Transactional schema migration, reopening after interruption, idempotence and
   a clear diagnostic when an older app opens a newer database. Preserve backups
   rather than destructively rewriting the only copy.
6. Protocol compatibility. Mandatory author authentication is a security boundary
   change, even if signature fields are additive. Plan a coordinated protocol
   bump and an explicit version mismatch for protocol-6 peers; never fall back to
   unsigned acceptance for compatibility. A downgrade must not reopen ingestion.

Do not remove public forwarding, erase history, replace peer identities, introduce
a privileged host, or claim legacy records are authenticated merely to simplify
the patch. If requirements conflict, explain the migration tradeoff before
committing to a product-visible change.

## Acceptance tests

Use independent Alice/Bob/Carol keys and real serialized messages, not only helper
functions that bypass the production ingestion path.

- Bob's valid operation reaches Carol through Alice while Bob is disconnected
  from Carol; authorship verifies and duplicate delivery is a no-op.
- Alice cannot create Bob's post, profile change, edit or delete, including when
  Alice signs using her own key and labels it with Bob's UUID.
- Mutating each signed field independently fails, including opaque payload bytes,
  channel/recipient, author, sequence, time, kind and signing-version/epoch fields.
- Missing, malformed, wrong-key, unknown-author and unsupported-version envelopes
  fail closed. A relay cannot establish or replace an author's trusted key.
- Removal, replacement and re-addition of a peer pin enforce the same trust and
  revocation policy before and after restart; historical verification keys cannot
  silently admit new operations from a removed peer.
- Invalid operations are neither stored nor rebroadcast, and cannot reserve a
  sequence key, advance a frontier, poison Lamport state, or affect unread counts.
- A forged-first/genuine-later pair leaves the genuine authenticated operation
  usable. Conflicting genuinely signed records remain visible collision errors.
- Broadcast, sync-response, reconnect and batched paths enforce the same rules;
  malformed members cannot bypass checks on the rest of a batch.
- Valid signed unknown operation kinds survive decode, persistence, reopening
  and relay byte-for-byte at the semantic-envelope level.
- DMs remain visible only to participants and are never publicly forwarded.
- Schema-2 fixtures include legacy posts, edits, deletes, profile changes, DM and
  topic history, gaps and malicious sequence/frontier claims. Migration preserves
  data, is transactional/repeatable and cannot let legacy data control signed state.
- Include a forged legacy row claiming the local UUID and unequal legacy prefixes
  exceeding `MAX_SEQ_VOORUIT` (1000). Neither may be auto-signed or prevent valid
  signed history from eventually synchronizing.
- Reconnect converges after migration with an offline author, a fresh peer, unequal
  legacy histories and a large signed history. Document any intentionally deferred
  recovery rather than accepting unsigned content to make a test pass.
- Protocol-6 peers receive a clear mismatch; old peers and downgrade attempts
  cannot create trusted operations. Identity and pinned keys survive restart.

Run focused adversarial tests first, then relevant workspace library/integration
checks and supported Windows/macOS builds. Keep crypto off the UI thread and
check bounded verification and sync behavior for realistic batches. Update
`SECURITY.md`, `docs/SECURITY-AUDIT.md`, architecture, validation and release notes
only to reflect protections actually implemented and tested. Retain all unrelated
security and licensing limitations.

## Claude implementation prompt

Read this entire document, the current security policy/audit and architecture, and
applicable local contributor instructions. Implement the complete author-
authentication repair described here. Use a subagent for independent review of
key binding, unsigned-history migration and adversarial tests. Start by tracing
all production ingestion paths and writing a concrete wire/storage/trust migration
design; resolve product-visible migration tradeoffs before destructive changes.
Then implement and test the complete path. Do not ship an unsigned fallback or
claim a partial signature patch fixes the vulnerability. Report changed behavior,
compatibility requirements, exact checks run and remaining limitations. Keep
private keys, runtime data, personal history and local assistant files outside
Git; do not publish releases as part of this remediation task.
