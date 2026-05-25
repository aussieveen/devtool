# Jira Tickets

Track a personal list of Jira tickets without leaving the terminal. Each ticket shows its ID, title, colour-coded status, and assignee. Tickets are persisted to disk and restored on next launch.

## Using Jira Tickets

Navigate to Jira Tickets using `[↑↓]` in the tool list and press `[→]` or `[enter]` to focus it.

### Key bindings

| Key | Action |
|---|---|
| `[↑↓]` | Navigate the ticket list |
| `[a]` | Open the add ticket input |
| `[x]` | Remove the selected ticket |
| `[shift+↑]` / `[shift+↓]` | Move the selected ticket up or down |
| `[o]` | Open the selected ticket in your browser |
| `[←]` | Return to the tool list |
| `[2]` | Open the config panel |
| `[q]` / `[esc]` | Quit |

### Adding a ticket

Press `[a]` to open the inline input. Type the Jira ticket ID (e.g. `ABC-123`) and press `[enter]` to add it. Press `[esc]` to cancel without adding. The ticket details are fetched immediately after adding.

### Removing a ticket

Navigate to a ticket with `[↑↓]` and press `[x]` to remove it from your list.

### Reordering tickets

Navigate to a ticket and use `[shift+↑]` or `[shift+↓]` to move it within the list.

### Opening in browser

Press `[o]` on a selected ticket to open it in your default browser using your configured Jira URL.

### Persistence

Your ticket list is saved to `~/.devtool/persistence.yaml` whenever it changes and is automatically restored on the next launch.

### Auto-refresh

Ticket data (status, assignee, title) is refreshed automatically every 15 minutes. Refresh events and any errors are logged to `[3]` Logs. The Activity feed shows a notification when a ticket's status changes.

## Configuring Jira

Press `[2]` to open the config panel, then press `[→]` on the Jira entry to open its settings.

Press `[e]` to open the configuration form:

| Field | Description |
|---|---|
| URL | Your Jira instance base URL (e.g. `https://yourcompany.atlassian.net`) |
| Email | The email address associated with your Jira account |
| API token | A Jira API token (generated at [id.atlassian.com](https://id.atlassian.com/manage-profile/security/api-tokens)) |

Trailing slashes are stripped from the URL automatically on save. The API token is masked in the config display. Press `[enter]` to save or `[esc]` to cancel.

### Enabling / disabling the tool

Press `[enter]` on the Jira entry in the config panel to toggle it on or off. A `☑` indicates it is enabled.
