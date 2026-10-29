# Privacy and local data

FitCom has no central account or chat-history service. Each installation stores
data locally and synchronizes with admitted peers. Deleting your local copy does
not remove others' copies, backups, or exports.

Invite codes contain public identity keys, peer IDs, display names, and network
addresses. Peers can learn your IP addresses and presence. Messages, shared files,
streams, typing status, and optional activity/music information disclose data to
their recipients. Encryption protects transport, not against recipients.
Approved remote control also shares clipboard text with the controller. Sensitive
text copied during that session, including passwords, can therefore be disclosed.

## Local storage

Use `--data-dir` for an explicit location. Windows portable mode uses a `data`
folder beside the executable; otherwise FitCom uses the OS application-data
directory. Existing installations retain their previous data location.

The directory includes `config.toml`, `identity.toml` (private identity),
`chat.sqlite`, `adressen.json`, caches, files, recordings, and logs. FitCom does not
encrypt these at rest. Protect your OS account and backups. Do not clone one
identity between running peers.

Logs may contain display/device names, file paths, peer identifiers, and IPs.
Inspect and redact before sharing. Debug logging exposes more detail.

## External connections

| Destination | Purpose | Data visible to service |
| --- | --- | --- |
| Your router | Optional UPnP/PCP/NAT-PMP mappings | Local device and mapping requests |
| GitHub release hosting/CDN | Windows updates | IP, time, application user agent |
| YouTube oEmbed and `i.ytimg.com` | Link titles/thumbnails | IP and video ID |
| `nytimes.com` | Daily Wordle puzzle | IP and requested date |

The app sends no personal GitHub token or NYT API key. External previews and puzzles
are additional connections beyond peer traffic; failures do not prevent core chat
or voice. Availability can change independently. FitCom adds no analytics or crash
report uploads. Links and optional system integrations can invoke other apps.
