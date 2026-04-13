#!/usr/bin/env bash
# Sets up a system-wide git commit-msg hook using convco.
# Run once — applies to every git repository on this machine.

set -euo pipefail

HOOKS_DIR="${HOME}/.config/git/hooks"
HOOK_FILE="${HOOKS_DIR}/commit-msg"

# Check for convco
if ! command -v convco &>/dev/null; then
  echo "convco is not installed. Install it with:"
  echo "  cargo install convco"
  exit 1
fi

# Create the global hooks directory
mkdir -p "$HOOKS_DIR"

# Write the commit-msg hook
cat > "$HOOK_FILE" <<'EOF'
#!/usr/bin/env bash
convco commit --check "$1"
EOF

chmod +x "$HOOK_FILE"

# Point git to the global hooks directory
git config --global core.hooksPath "$HOOKS_DIR"

echo "✅ System-wide commit-msg hook installed at ${HOOK_FILE}"
echo "   All git repositories on this machine will now enforce Conventional Commits."
echo ""
echo "   Supported types that trigger a release:"
echo "     feat:     → minor version bump"
echo "     fix:      → patch version bump"
echo "     perf:     → patch version bump"
echo "     revert:   → patch version bump"
echo "     <type>!:  → major version bump (breaking change)"
echo ""
echo "   Non-releasable types (no release created):"
echo "     build: chore: ci: docs: style: test: refactor:"
