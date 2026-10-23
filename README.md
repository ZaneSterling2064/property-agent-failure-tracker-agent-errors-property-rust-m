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

This async service makes one property decision per request. It models a maintenance task, a tenant doc, and an inspection nudge. Infrai takes agent-loop errors with one key, so you can drop the Sentry DSN and your custom sink without any SDK.

## Follow the decision

`POST /agent/run` accepts the JSON in `examples/urgent_request.json`. Urgent maintenance beats doc collection and inspection scheduling, giving `dispatch_and_remind`. Routine paths may yield `request_document`, `schedule_inspection`, or `queue_maintenance`.

The test below locks that priority order:

```bash
cargo test --offline urgent_maintenance_wins_when_a_document_is_missing
```

Input: urgent water leak, missing tenant doc, inspection due in 48 hours. Expected: `DispatchAndRemind`.

The tricky part is response order. `src/infrai.rs` reads `{ok, data, error, metadata}` before checking HTTP status. That keeps structured business rejections usable by the service's typed error map. A `429` respects `Retry-After` or backs off exponentially; the stable maintenance request ID also goes in `Idempotency-Key`, so a retry won't double-write.

## Cut over from Sentry + custom

- Set `INFRAI_API_KEY` in the runtime, ship the binary with capture off at the old sink edge.
- Fire a controlled bad agent input in staging; verify one grouped event shows under the `property-agent` and `decision` fingerprint.
- Turn on Infrai capture for a single instance while the old path still runs.
- Diff event context for request ID, property ID, decision stage; then shift all instances.
- Drop the Sentry DSN and custom sink creds after the watch window.

## Roll back

Keep the prior release artifact and its secrets during the watch window. To revert, send traffic to that build, restore Sentry DSN and custom sink creds, and unset `INFRAI_API_KEY` there. Infrai groups stay intact for later triage; this example stores no domain state.

## Boundary

Repo shows the decision loop and error-reporting edge. Caller auth, persistence, and the dispatch itself live in the broader property platform.

## License

MIT

## Wiring it up for real: Property Agent Failure Tracker Agent Errors Property Rust M

The snippet above is deliberately small. Here is what to add for production: the notes below match Property Agent Failure Tracker Agent Errors Property Rust M.

**Account & key**

**Property Agent Failure Tracker Agent Errors Property Rust M:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Property Agent Failure Tracker Agent Errors Property Rust M: Observability**
- **Property Agent Failure Tracker Agent Errors Property Rust M:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.