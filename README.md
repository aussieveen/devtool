# Development Tool

A terminal UI built with [Ratatui] for day-to-day developer workflows — checking service health, generating auth tokens, and tracking Jira tickets, all without leaving the terminal.

[Ratatui]: https://ratatui.rs

## Installation
### Prebuilt Binaries (Recommended)

Prebuilt binaries are available for macOS and Linux.
1. Go to the [latest release](https://github.com/aussieveen/devtool/releases/latest) of this repository.
2. Download the archive corresponding to your operating system.
3. Extract the archive.
4. Move the dev-tool binary into a directory on your `PATH`, for example:
    ```bash
    sudo mv dev-tool /usr/local/bin
    ```
5. Run `dev-tool` from the command line in order to bring up the TUI.
### Run From Source

If you have Rust installed, you can also run the application directly from source.

```bash
git clone git@github.com:aussieveen/devtool.git
cd devtool
cargo run
```

### Supported Platforms
- macOS (Intel & Apple Silicon)
- Linux (x86_64)

### Notes

Ensure the binary is executable: `chmod +x dev-tool`

Make sure `/usr/local/bin` (or your chosen directory) is included in your PATH.

#### macOS: allowing the binary to run

macOS blocks binaries downloaded from the internet until you explicitly allow them. After trying to run `dev-tool` for the first time, macOS will show a security alert.

To allow it:

1. Open **System Settings → Privacy & Security**
2. Scroll down to the Security section
3. Click **Open Anyway** next to the `dev-tool` entry
4. Confirm by clicking **Open** in the prompt that follows

Alternatively, remove the quarantine flag from the terminal before running:

```bash
xattr -d com.apple.quarantine dev-tool
```

## Navigation

The UI has three panels, switchable at any time with `[1]`, `[2]`, and `[3]`:

- **`[1]` Tool panel** — the active tool's content (service grid, token generator, ticket list)
- **`[2]` Config panel** — inline configuration for the active tool
- **`[3]` Logs panel** — Activity feed and App Log (see [Logs](#logs) below)

The **footer** is the key legend. It updates contextually based on what is focused:
- **Line 1** — global navigation hints and `[q/esc] Quit`
- **Line 2** — item-specific actions, shown only when relevant (e.g. when a row is selected)

## Features

### Service Status

Colour-coded commit status grid across staging, preproduction, and production. Auto-scans every 15 minutes. Generates a compare URL when environments diverge.

→ [Usage & configuration guide](docs/service-status.md)

```
┌──────────────────────────┬──────────────────────────────────────────────────────────────┐
│ [1] Tools                │ Service Status                                               │
│                          │                                                              │
│ ▶ Service Status         │  Service            Staging      Preproduction   Production  │
│   Token Generator        │  ─────────────────────────────────────────────────────────   │
│   Jira Tickets           │  ▍  api-gateway     a1b2c3d      a1b2c3d         e4f5g6h     │
│                          │  ▍  auth-service    9x8y7z6      1a2b3c4         1a2b3c4     │
│ [2] Config               │  ▍  user-service    …            …               …           │
│   ☑ Service Status       │                                                              │
│   ☐ Token Generator      │  ▍ Up to date   ▍ New version in pipeline                    │
│                          │  ▍ Pending production deployment   ▍ Requires maintenance    │
│ [3] Logs                 │                                                              │
│   Activity               │                                                              │
│   App Log                │                                                              │
└──────────────────────────┴──────────────────────────────────────────────────────────────┘
 ──────────────────────────────────────────────────────────────────────────────────────────
 [↑↓] Navigate  [s] Scan  [←] Tool list  [2] Config  [3] Logs  [q/esc] Quit
 [o] Open in browser  [c] Copy url
```

### M2M Auth0 Token Generator

Generate M2M tokens on demand per service and environment. Tokens are copied to clipboard with a single keystroke.

→ [Usage & configuration guide](docs/token-generator.md)

```
┌──────────────────────────┬──────────────────────────────────────────────────────────────┐
│ [1] Tools                │ Token Generator                                              │
│                          │                                                              │
│   Service Status         │  ┌ Services ──────────────┐ ┌ Environments ───────────────┐  │
│ ▶ Token Generator        │  │                        │ │                             │  │
│   Jira Tickets           │  │ ▶ payment-service      │ │ ▶ [✓] development           │  │
│                          │  │   auth-service         │ │   […] staging               │  │
│ [2] Config               │  │                        │ │   [ ] production            │  │
│   ☐ Service Status       │  └────────────────────────┘ └─────────────────────────────┘  │
│   ☑ Token Generator      │                                                              │
│                          │                                                              │
│ [3] Logs                 │                                                              │
│   Activity               │                                                              │
│   App Log                │                                                              │
└──────────────────────────┴──────────────────────────────────────────────────────────────┘
 ──────────────────────────────────────────────────────────────────────────────────────────
 [←→] Switch panel  [↑↓] Navigate  [return] Generate  [2] Config  [q/esc] Quit
 [c] Copy token
```

### Jira Tickets

Track Jira tickets by ID — title, status, and assignee displayed inline. Persisted to disk across restarts. Auto-refreshes every 15 minutes.

→ [Usage & configuration guide](docs/jira.md)

```
┌──────────────────────────┬──────────────────────────────────────────────────────────────┐
│ [1] Tools                │ Jira Tickets                                                 │
│                          │                                                              │
│   Service Status         │  ▶ ABC-123 - Fix authentication bug in login flow            │
│   Token Generator        │    In Progress   @john.doe                                   │
│ ▶ Jira Tickets           │                                                              │
│                          │    ABC-456 - Update API rate limiting documentation          │
│ [2] Config               │    In Review   @jane.smith                                   │
│   ☐ Service Status       │                                                              │
│   ☑ Jira Tickets         │    ABC-789 - Optimise database query performance             │
│                          │    Done   @mike.jones                                        │
│ [3] Logs                 │                                                              │
│ ● Activity               │                                                              │
│   App Log                │                                                              │
└──────────────────────────┴──────────────────────────────────────────────────────────────┘
 ──────────────────────────────────────────────────────────────────────────────────────────
 [↑↓] Navigate  [a] Add ticket  [←] Tool list  [2] Config  [q/esc] Quit
 [x] Remove  [o] Open in browser  [shift+↑↓] Move
```

### Logs

Press `[3]` from anywhere to open the Logs panel. It has two sub-sections, switchable with `[↑↓]`:

- **Activity** — human-readable summaries of changes (new deployment, status change, ticket added). A `●` dot appears in the sidebar when there are unread entries.
- **App Log** — structured log entries (timestamp, severity, source, message). Severity levels follow [RFC 5424](https://datatracker.ietf.org/doc/html/rfc5424): `[ERROR]`, `[WARN]`, `[INFO]`, `[DEBUG]`. All log data is in-memory only and cleared on restart.

### Configuration

Each tool is configured inline via the `[2]` panel — no separate windows or prompts. Press `[2]` to open it, then `[→]` on a tool to enter its settings, or `[enter]` to toggle it on/off. Press `[←]` to return.

### Persistence

Jira ticket selections are saved to `~/.devtool/persistence.yaml` and restored on the next launch. All other tool state is in-memory for the duration of the session.

## License

Copyright (c) Simon McWhinnie <simon.mcwhinnie@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
