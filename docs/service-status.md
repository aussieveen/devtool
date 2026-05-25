# Service Status

Displays a colour-coded commit grid showing the deployed commit hash for each service across staging, preproduction, and production. When preproduction and production diverge, it generates a compare URL so you can see exactly what's pending release.

## Using Service Status

Navigate to Service Status using `[↑↓]` in the tool list and press `[→]` or `[enter]` to focus it.

### Key bindings

| Key | Action |
|---|---|
| `[↑↓]` | Navigate the service list |
| `[s]` | Trigger an immediate scan |
| `[o]` | Open the compare URL in your browser (when available) |
| `[c]` | Copy the compare URL to clipboard (when available) |
| `[←]` | Return to the tool list |
| `[2]` | Open the config panel |
| `[3]` | Open the logs panel |
| `[q]` / `[esc]` | Quit |

### Commit status grid

Each cell shows the short commit hash for that environment. The colour of the status bar on the left of each row indicates the deployment state:

| Colour | Meaning |
|---|---|
| Green | Up to date — all environments match |
| Blue | New version in pipeline — staging has a newer commit |
| Yellow | Pending production deployment — preproduction and production differ |
| Red | Requires maintenance — check the App Log for details |

A `…` in a cell means the commit hash is still being fetched.

### Compare URL

When preproduction and production have different commits, a compare URL is generated using the repository URL configured for that service. Use `[o]` to open it or `[c]` to copy it.

### Auto-scan

The service grid is scanned automatically every 15 minutes. Press `[s]` at any time to trigger an immediate scan. Scan results and any errors are logged to `[3]` Logs.

## Configuring Service Status

Press `[2]` to open the config panel, then press `[→]` on the Service Status entry to open its settings.

### Adding a service

Press `[a]` to open the add service form. Fill in the fields using `[tab]`, `[↑↓]`, or `[↓]` to move between them:

| Field | Description |
|---|---|
| Name | Display name for the service |
| Staging | Health check URL for the staging environment |
| Preproduction | Health check URL for the preproduction environment |
| Production | Health check URL for the production environment |
| Repository | Repository base URL, used to generate compare links |

Press `[enter]` to save or `[esc]` to cancel.

### Editing a service

Navigate to a service with `[↑↓]` and press `[e]` to open the edit form. The fields are pre-filled with the current values. Press `[enter]` to save or `[esc]` to cancel.

### Removing a service

Navigate to a service with `[↑↓]` and press `[x]` to remove it. The removal takes effect immediately.

### Enabling / disabling the tool

Press `[enter]` on the Service Status entry in the config panel to toggle it on or off. A `☑` indicates it is enabled.
