# another-it-tg-bridge

Posts new articles from [another-it.ru](https://another-it.ru/) to the [another_it](https://t.me/another_it) Telegram channel.

## Quick Start

1. Clone the repository and enter it:
   ```sh
   git clone https://github.com/qqrm/another-it-tg-bridge.git
   cd another-it-tg-bridge
   ```
2. In **Settings → Secrets and variables → Actions**, add:
   - `TELEGRAM_BOT_TOKEN` – Telegram bot token.
   - `TELEGRAM_CHAT_ID` – target chat identifier.
   - `GOOGLE_CSE_API_KEY` – Google Custom Search API key.
   - `GOOGLE_CSE_CX` – Programmable Search Engine ID.
   - (optional) `DEV_TELEGRAM_CHAT_ID` – debug chat identifier for manual backfills.
3. Enable GitHub Actions for the repository if it is disabled.
4. Trigger the workflow: open the **Actions** tab, select **Post to Telegram**, and click **Run workflow** (`workflow_dispatch`).

## Environment variables

- `TELEGRAM_BOT_TOKEN` – Telegram bot token.
- `TELEGRAM_CHAT_ID` – target chat identifier.
- `GOOGLE_CSE_API_KEY` – Google Custom Search API key.
- `GOOGLE_CSE_CX` – Programmable Search Engine ID.
- `RUST_LOG` – optional logging level (e.g., `info`).

## Manual run

```sh
export TELEGRAM_BOT_TOKEN=...
export TELEGRAM_CHAT_ID=...
export GOOGLE_CSE_API_KEY=...
export GOOGLE_CSE_CX=...
cargo run --release
```

Pass an optional date argument (`YYYY/MM/DD` or `YYYY-MM-DD`) to probe a single day via Google CSE:

```sh
cargo run --release -- 2026-03-28
```

Pass `--days N` to scan a trailing window including today:

```sh
cargo run --release -- --days 14
```

## GitHub Actions

The workflow at `.github/workflows/post.yml` downloads the release binary on a schedule or manual dispatch and checkpoints each send in the delivery-state branch. Scheduled runs query Google CSE for the recent date prefixes to absorb indexing delays while still deduplicating already-sent links.
The manual workflow at `.github/workflows/debug.yml` targets `DEV_TELEGRAM_CHAT_ID` when present and defaults to a 14-day backfill into an isolated debug state file.
Rust toolchain updates are handled by Dependabot with auto-merge enabled for its pull requests.

## Development

Ensure the required development tools (`rustfmt`, `clippy`, `cargo-machete`, and `wrkflw`) are installed via `rustup` and `cargo` as needed.

## Origins

This project was bootstrapped from [`rust-hh-feed`](https://github.com/qqrm/rust-hh-feed) and retains a similar layout and Actions configuration for maintainers familiar with that repository.

## Delivery checkpoints and recovery

Production delivery stores its checkpoint in the separate `delivery-state` branch using GitHub's contents API. The workflow token is scoped to the restore/send steps and consumed by the checkpoint helper; checkout does not persist credentials. The remote file is a versioned JSON envelope containing base64-encoded application state and a `pending` SHA-256 digest. No token, chat ID, or API response is stored.

Before each send, a pending operation is committed. Only an acknowledged send promotes the application state and clears pending in the same API write. Authentication, network, corrupt-state and checkpoint conflicts stop delivery. A missing branch is the only first-run condition; existing legacy state is migrated where available. A missing file in an existing state branch is an error. Caches and run artifacts are no longer the authority after migration.

A timeout, process crash, or failed checkpoint after an acknowledgement leaves pending set. Subsequent scheduled runs stop before sending. This provides safe interruption, not exactly-once delivery: Telegram has no idempotency key for this operation.

To reconcile:

1. Pause the production workflow and inspect the failed run and Telegram channel. Do not blindly retry or remove state.
2. Read the checkpoint file on `delivery-state` and decode its `state` field locally. If the pending message was delivered, add its confirmed entry to the application state. If it was definitely not delivered, retain the previous application state. If the outcome is still unknown, leave pending intact.
3. Encode the reconciled state, set `pending` to null, and update that same file with its current blob SHA to reject concurrent changes. Keep tokens, chat IDs and response bodies out of the commit. Record the evidence in the recovery change.
4. For local runs, update the local state consistently and remove only its `.pending` sidecar after reconciliation. Resume the workflow. Already confirmed entries are skipped.

## Dependency and workflow policy

Cargo and Actions updates share a weekly Dependabot maintenance batch with a seven-day cooldown; security fixes use a dedicated group. Rust toolchain updates remain tracked separately. Native auto-merge requires the exact PR head to pass `ci-gate` and `workflow-security-gate`, with no review requirement. Failed or skipped required jobs cannot pass a gate. The privileged controller reads metadata only; it never checks out PR code or consumes PR artifacts.

Existing release and delivery triggers are preserved. A merge performed using `GITHUB_TOKEN` does not guarantee a follow-up push workflow. Guaranteed automatic post-merge releases need a separate GitHub App token or verified release controller; this change does not claim that coverage.

The confirmed application state is a sorted list of canonical article URLs. Titles are sent as plain text, including literal `<` and `&`. To reconcile a delivered article, add its URL to that list before clearing pending.

The release publishes `delivery-manifest.json` (protocol 1, source commit, SHA-256) alongside the binary. Posting verifies both protocol and hash before execution. Missing old manifests and asset-update races fail closed; wait for the normal release to finish before retrying.

Every mutation compares the remote blob SHA with the snapshot recorded at restore (or the prior successful write). A stale runner cannot replace newer progress, even across branches; a SHA conflict stops before another send.
