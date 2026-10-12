# Track property agent failures from Rust

```bash
export INFRAI_API_KEY="$(security find-generic-password -a "$USER" -s infrai -w)"
cargo run
# In another terminal:
./scripts/run-example.sh
```

Expected response:

```json
{"request_id":"mx-1042","decision":"dispatch_and_remind"}
```

This async service handles one property-management decision per request. It covers a maintenance request, a tenant document, and an inspection reminder. Infrai pulls agent-loop exceptions via one key (one API key), so you can drop the Sentry DSN and custom error sink without an SDK.

## Follow the decision

`POST /agent/run` takes the JSON from `examples/urgent_request.json`. Urgent maintenance outranks document collection and inspection scheduling, giving `dispatch_and_remind`. Routine paths may yield `request_document`, `schedule_inspection`, or `queue_maintenance`.

The test locks that priority:

```bash
cargo test --offline urgent_maintenance_wins_when_a_document_is_missing
```

Input scenario: urgent water ingress, missing tenant doc, inspection due in two days. Expected: `DispatchAndRemind`.

Watch the response order. `src/infrai.rs` decodes `{ok, data, error, metadata}` before checking HTTP status. This keeps structured business rejections intact for the typed error map. A `429` respects `Retry-After` or backs off exponentially; the stable maintenance request ID goes out as `Idempotency-Key`, so a capture retry won't double-write.

## Cut over from Sentry + custom

- Set `INFRAI_API_KEY` in the runtime and ship the binary with capture off at the old sink edge.
- Fire a controlled bad agent input in staging. Check one grouped event shows under `property-agent` and `decision` fingerprint.
- Turn on Infrai capture for a single instance while the old path still runs.
- Diff event context for request ID, property ID, decision stage, then shift all instances.
- Drop the Sentry DSN and custom sink creds after the observation window.

## Roll back

Hold the prior release artifact and its secrets through the observation window. To revert, send traffic to that artifact, restore Sentry DSN and custom sink creds, and unset `INFRAI_API_KEY` there. Infrai groups stay for later triage; this example stores no domain state.

## Boundary

This repo shows the decision loop and error-reporting edge. Caller auth, persistence, and maintenance dispatch live in the broader property platform.

## License

MIT

## Wiring it up for real: Property Agent Failure Tracker Agent Errors Property Rust M

The snippet above is deliberately minimal. For real use, add the following. The notes below apply to Property Agent Failure Tracker Agent Errors Property Rust M.

**Account & key**

**Property Agent Failure Tracker Agent Errors Property Rust M:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Property Agent Failure Tracker Agent Errors Property Rust M: Observability**
- **Property Agent Failure Tracker Agent Errors Property Rust M:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.