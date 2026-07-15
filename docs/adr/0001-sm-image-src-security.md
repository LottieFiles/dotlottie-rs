# State machine image slot actions: reject data: URIs, leave remote URLs ungated

Status: accepted (2026-07-09)

The `SetImageSlot` state machine action resolves its `src` like theme image
rules do (`http(s)://` → remote, else package `i/` path) — but with two
deliberate asymmetries that will otherwise look arbitrary in the engine code:

1. **`data:` URIs are rejected**, at runtime, on the *resolved* value (after
   `$input` reference resolution), making the action a no-op. Themes accept
   `data:` in the same `src` field. We rejected them in state machine actions
   to keep multi-megabyte base64 blobs out of state machine definitions, and
   enforce at runtime (not just schema level) because a `$`-referenced String
   input could otherwise smuggle one past an authoring-time check. State
   machine `src` semantics are intentionally a strict subset of theme `src`
   semantics.

2. **Remote `http(s)` URLs are NOT whitelist-gated**, even though `OpenUrl` —
   the other action reaching outside the file — is default-deny behind a
   host-configured whitelist. The distinction is navigation vs asset loading:
   the format already loads remote images ungated (theme image rules), so a
   state-machine-triggered fetch adds no capability a file didn't already have
   via `SetTheme`; gating only the action would be bypassable security
   theater, and default-deny would break remote-image files on every
   unconfigured host. Fetch policy belongs to the host environment (e.g. CSP
   on web).

Full alternatives analysis:
[docs/spec-updates/slot-actions-alternatives.md](../spec-updates/slot-actions-alternatives.md)
(A5, A6).
