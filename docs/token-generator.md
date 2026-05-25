# M2M Auth0 Token Generator

Generates machine-to-machine Auth0 access tokens on demand. Select a service and an environment, press `[enter]`, and the token is fetched and ready to copy in seconds.

## Using the Token Generator

Navigate to Token Generator using `[↑↓]` in the tool list and press `[→]` or `[enter]` to focus it.

The tool panel has two lists side by side: **Services** on the left and **Environments** on the right.

### Key bindings

| Key | Action |
|---|---|
| `[↑↓]` | Navigate within the focused list |
| `[←→]` | Switch focus between Services and Environments |
| `[enter]` | Generate a token for the selected service + environment |
| `[c]` | Copy the generated token to clipboard |
| `[←]` (from Services) | Return to the tool list |
| `[2]` | Open the config panel |
| `[q]` / `[esc]` | Quit |

### Token state indicators

Each environment row shows a status indicator for its current token:

| Indicator | Meaning |
|---|---|
| `[ ]` | Idle — no token generated yet |
| `[…]` | Generating — request in progress |
| `[✓]` | Ready — token available, press `[c]` to copy |
| `[x]` | Error — request failed, check `[3]` Logs for details |

Press `[enter]` again on an errored environment to retry.

## Configuring the Token Generator

Press `[2]` to open the config panel, then press `[→]` on the Token Generator entry to open its settings.

The config is a single unified list. The first row is always **Auth0 Endpoints** — your shared Auth0 base URLs. Below that are your configured services, each with its own client credentials per environment.

Navigate with `[↑↓]`.

### Configuring Auth0 endpoints

Select the **Auth0 Endpoints** row (the first row) and press `[e]` to edit. Fill in the base URL for each environment:

| Field | Description |
|---|---|
| Local | Auth0 token endpoint base URL for local development |
| Staging | Auth0 token endpoint base URL for staging |
| Preproduction | Auth0 token endpoint base URL for preproduction |
| Production | Auth0 token endpoint base URL for production |

Trailing slashes are stripped automatically on save. Press `[enter]` to save or `[esc]` to cancel.

### Adding a service

Press `[a]` from anywhere in the config list to open the add service form. Fill in the service details:

| Field | Description |
|---|---|
| Name | Display name for the service |
| Local client ID | Auth0 client ID for local development |
| Local client secret | Auth0 client secret for local development |
| Staging client ID | Auth0 client ID for staging |
| Staging client secret | Auth0 client secret for staging |
| Preproduction client ID | Auth0 client ID for preproduction |
| Preproduction client secret | Auth0 client secret for preproduction |
| Production client ID | Auth0 client ID for production |
| Production client secret | Auth0 client secret for production |

Use `[tab]`, `[↑↓]`, or `[↓]` to move between fields. Press `[enter]` to save or `[esc]` to cancel.

### Editing a service

Navigate to a service row with `[↑↓]` and press `[e]` to edit it. The form is pre-filled with the current values. Press `[enter]` to save or `[esc]` to cancel.

### Removing a service

Navigate to a service row and press `[x]` to remove it.

### Enabling / disabling the tool

Press `[enter]` on the Token Generator entry in the config panel to toggle it on or off. A `☑` indicates it is enabled.

> **Note:** The tool requires at least one service to be configured before it will show in the tool panel.
