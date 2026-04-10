# Development Tool

A terminal UI built with [Ratatui] for day-to-day developer workflows — checking service health, generating auth tokens, and tracking Jira tickets, all without leaving the terminal.

[Ratatui]: https://ratatui.rs

## Navigation

The UI has two panels, switchable at any time with `[1]` and `[2]`:

- **`[1]` Tool panel** — the active tool's content (service grid, token generator, ticket list)
- **`[2]` Config panel** — inline configuration for the active tool

The **footer** is the key legend. It updates contextually based on what is focused:
- **Line 1** — global navigation hints and `[q/esc] Quit`
- **Line 2** — item-specific actions, shown only when relevant (e.g. when a row is selected)

## Features

### Service Status

- Configure each service with **staging**, **preproduction**, and **production** health check URLs, plus a repository URL — all via the `[2]` config panel.
- Displays a colour-coded commit status grid showing deployed commits across all environments.
- Generates a **compare URL** between preproduction and production when they diverge, ready to open in your browser or copy to clipboard.
- Auto-scans every **15 minutes** to keep status current.

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
└──────────────────────────┴──────────────────────────────────────────────────────────────┘
 ──────────────────────────────────────────────────────────────────────────────────────────
 [↑↓] Navigate  [s] Scan  [←] Tool list  [2] Config  [q/esc] Quit
 [o] Open in browser  [c] Copy url
```

### M2M Auth0 Token Generator

- Configure Auth0 endpoints and service credentials via the `[2]` config panel — edited inline, no popups.
- Select a **service** and **environment** to generate a token on demand.
- Token state is shown with status indicators: `[ ]` idle, `[…]` generating, `[✓]` ready, `[x]` error.
- Copy the generated token to clipboard with a single keystroke.

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
└──────────────────────────┴──────────────────────────────────────────────────────────────┘
 ──────────────────────────────────────────────────────────────────────────────────────────
 [←→] Switch panel  [↑↓] Navigate  [return] Generate  [2] Config  [q/esc] Quit
 [c] Copy token
```

### Jira Tickets

- Add tickets by **Jira ID** using an inline input (press `[a]`, type the ID, press `[enter]`).
- Displays each ticket's ID, title, colour-coded status, and assignee.
- Remove and reorder tickets to suit your workflow.
- Ticket data is **persisted to disk** (`~/.devtool/persistence.yaml`) and restored on next launch.
- Auto-refreshes every **15 minutes**.

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
└──────────────────────────┴──────────────────────────────────────────────────────────────┘
 ──────────────────────────────────────────────────────────────────────────────────────────
 [↑↓] Navigate  [a] Add ticket  [←] Tool list  [2] Config  [q/esc] Quit
 [x] Remove  [o] Open in browser  [shift+↑↓] Move
```

### Configuration

Each tool's settings are edited inline via the `[2]` config panel — no separate windows or prompts.

- **Service Status** — add, edit, and remove services with name and environment URLs
- **Token Generator** — configure Auth0 endpoints and per-service credentials
- **Jira** — set your Jira URL, email, and API token (token is masked in display)

Navigate to the config panel with `[2]`, select a tool with `[enter]` to enable it, or press `[→]` to open its settings. Press `[←]` to return.

### Persistence

- All tool data is retained while the TUI is running — navigating between tools does not reset their state.
- Jira ticket selections are saved to `~/.devtool/persistence.yaml` and automatically restored on the next launch.

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
5. Create the configuration directory and copy the default configuration:
    ```bash
    mkdir -p ~/.devtool
    cp config/config.yaml.dist ~/.devtool/config.yaml
    ```
6. Open `~/.devtool/config.yaml` and configure it with your required values.
7. Run `dev-tool` from the command line in order to bring up the TUI.
### Run From Source

If you have Rust installed, you can also run the application directly from source.

```bash
git clone git@github.com:aussieveen/devtool.git
cd devtool
cargo run
```

⚠️ The configuration file at `~/.devtool/config.yaml` is still required when running from source.

### Supported Platforms
- macOS (Intel & Apple Silicon)
- Linux (x86_64)

### Notes
Ensure the binary is executable: `chmod +x dev-tool`

Make sure `/usr/local/bin` (or your chosen directory) is included in your PATH.

## License

Copyright (c) Simon McWhinnie <simon.mcwhinnie@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
