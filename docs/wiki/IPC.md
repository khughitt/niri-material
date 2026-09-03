You can communicate with the running niri instance over an IPC socket.
Check `niri msg --help` for available commands.

The `--json` flag prints the response in JSON, rather than formatted.
For example, `niri msg --json outputs`.

> [!TIP]
> If you're getting parsing errors from `niri msg` after upgrading niri, make sure that you've restarted niri itself.
> You might be trying to run a newer `niri msg` against an older `niri` compositor.

### Event Stream

<sup>Since: 0.1.9</sup>

While most niri IPC requests return a single response, the event stream request will make niri continuously stream events into the IPC connection until it is closed.
This is useful for implementing various bars and indicators that update as soon as something happens, without continuous polling.

The event stream IPC is designed to give you the complete current state up-front, then follow up with updates to that state.
This way, your state can never "desync" from niri, and you don't need to make any other IPC information requests.

Where reasonable, event stream state updates are atomic, though this is not always the case.
For example, a window may end up with a workspace id for a workspace that had already been removed.
This can happen if the corresponding workspaces-changed event arrives before the corresponding window-changed event.

To get a taste of the events, run `niri msg event-stream`.
Though, this is more of a debug function than anything.
You can get raw events from `niri msg --json event-stream`, or by connecting to the niri socket and requesting an event stream manually.

You can find the full list of events along with documentation [here](https://niri-wm.github.io/niri/niri_ipc/enum.Event.html).

### Window signals

`set-window-signal`, `pulse-window-signal`, and `clear-window-signal` write
per-window signal state. They return `Handled` on success and an error reply
without changing state on failure.

```sh
niri msg set-window-signal --id 12 --source familiar \
    --accent '#e5a33c' --level demand --motion pulse --tag cats/ginger \
    --ttl-ms 30000 --after-level quiet --after-motion breathe --until-focus
niri msg pulse-window-signal --id 12 --source familiar --kind done
niri msg clear-window-signal --id 12 --source familiar
```

| Request | Required flags | Optional flags and defaults |
| --- | --- | --- |
| `set-window-signal` | `--id`, `--source` | `--accent`; `--level` (`quiet`); `--motion` (`static`); `--tag`; `--ttl-ms`; `--after-level`; `--after-motion`; `--until-focus` (off) |
| `pulse-window-signal` | `--id`, `--source`, `--kind` (`ping`, `done`, or `error`) | `--accent` |
| `clear-window-signal` | `--id`, `--source` | — |

`set-window-signal` replaces the source's whole slot. With `--ttl-ms`, at
least one `--after-level` or `--after-motion` is required; the other defaults
to `quiet` or `static`. Supplying either `after` flag without `--ttl-ms` is
an error. `--until-focus` demotes the slot to that after pair when the window
is focused. `pulse-window-signal` requires that the source already have a
slot; `clear-window-signal` removes its slot and impulses.

Sources are case-sensitive, `niri` is reserved, and a window accepts at most
16 external sources. Source names are limited to 64 bytes and tags to 256
bytes. `--ttl-ms` is at most 86400000 (24 hours). A window retains at most
four live impulses; a fifth evicts the oldest. Accent colors are `#rrggbb` or
`#rrggbbaa`; alpha is accepted but ignored. Unknown window or source,
reserved or oversized source, oversized tag or TTL, invalid TTL/after
combination, malformed color, and an invalid level, motion, or impulse kind
all return errors. Rejected requests are atomic.

`Window.signal` is either `null` or a folded signal. Event streams emit the
same value when it changes as `WindowSignalChanged`:

```json
{
  "WindowSignalChanged": {
    "id": 12,
    "signal": {
      "level": "Demand",
      "motion": "Pulse",
      "accent": "#e5a33c",
      "tag": "cats/ginger",
      "sources": ["familiar"],
      "impulses": [{
        "source": "familiar",
        "kind": "Done",
        "accent": null,
        "at": {"secs": 42, "nanos": 0},
        "expires_at": {"secs": 43, "nanos": 500000000}
      }]
    }
  }
}
```

The `Window` JSON object gains the same `"signal": { ... }` field (or
`"signal": null` when no source has a slot). Enum values use Rust spelling
on the wire: levels are `Quiet`, `Active`, `Notice`, `Demand`; motion is
`Static`, `Breathe`, `Pulse`, `Flash`; impulse kinds are `Ping`, `Done`,
`Error`. The CLI accepts their kebab-case equivalents. `at` and `expires_at`
are monotonic `Timestamp` objects with `secs` and `nanos` fields.

### Programmatic Access

`niri msg --json` is a thin wrapper over writing and reading to a socket.
When implementing more complex scripts and modules, you're encouraged to access the socket directly.

Connect to the UNIX domain socket located at `$NIRI_SOCKET` in the filesystem.
Write your request encoded in JSON on a single line, followed by a newline character, or by flushing and shutting down the write end of the connection.
Read the reply as JSON, also on a single line.

You can use `socat` to test communicating with niri directly:

```sh
$ socat STDIO "$NIRI_SOCKET"
"FocusedWindow"
{"Ok":{"FocusedWindow":{"id":12,"title":"t socat STDIO /run/u ~","app_id":"Alacritty","workspace_id":6,"is_focused":true}}}
```

The reply is an `Ok` or an `Err` wrapping the same JSON object as you get from `niri msg --json`.

For more complex requests, you can use `socat` to find how `niri msg` formats them:

```sh
$ socat STDIO UNIX-LISTEN:temp.sock
# then, in a different terminal:
$ env NIRI_SOCKET=./temp.sock niri msg action focus-workspace 2
# then, look in the socat terminal:
{"Action":{"FocusWorkspace":{"reference":{"Index":2}}}}
```

You can find all available requests and response types in the [niri-ipc sub-crate documentation](https://niri-wm.github.io/niri/niri_ipc/).

### Backwards Compatibility

The JSON output *should* remain stable, as in:

- existing fields and enum variants should not be renamed
- non-optional existing fields should not be removed

However, new fields and enum variants will be added, so you should handle unknown fields or variants gracefully where reasonable.

The formatted/human-readable output (i.e. without `--json` flag) is **not** considered stable.
Please prefer the JSON output for scripts, since I reserve the right to make any changes to the human-readable output.

The `niri-ipc` sub-crate (like other niri sub-crates) is *not* API-stable in terms of the Rust semver; rather, it follows the version of niri itself.
In particular, new struct fields and enum variants will be added.
