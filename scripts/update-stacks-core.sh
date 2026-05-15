#!/usr/bin/env bash
#
# Bump the stacks-network/stacks-core git revision used for the upstream
# canonical wire-format crates (stacks-codec, stacks-common, clarity, stackslib).
#
# Usage:
#   scripts/update-stacks-core.sh                 # pin to latest develop HEAD
#   scripts/update-stacks-core.sh <rev|tag|sha>   # pin to an explicit ref
#
# What this script does:
#   1. Resolves the requested ref to a full commit SHA via GitHub's API.
#   2. Rewrites every `rev = "..."` entry inside the [dependencies] block of
#      Cargo.toml that points at stacks-network/stacks-core.
#   3. Runs `cargo update -p stackslib -p clarity -p stacks-common -p stacks-codec`
#      so Cargo.lock is regenerated.
#   4. Optionally runs a `cargo check` to fail loudly if the pin doesn't build.
#
# Why does it have to be a script? The whole point of going through this is
# so we can delete the giant chunks of wire-format code copy-pasted into
# `src/stacks_tx`, `src/stacks_block`, `src/post_condition`, `src/clarity_value`,
# `src/address`, etc. The "real" copy now lives upstream in stacks-core; running
# this script is how we periodically sync to a newer upstream copy.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="$REPO_ROOT/Cargo.toml"

UPSTREAM_OWNER="stacks-network"
UPSTREAM_REPO="stacks-core"
UPSTREAM_URL="https://github.com/${UPSTREAM_OWNER}/${UPSTREAM_REPO}"

# Cargo package names whose rev we manage in Cargo.toml. Keep this in sync with
# the [dependencies] section.
MANAGED_PACKAGES=(stacks-codec stacks-common clarity stackslib)

target_ref="${1:-develop}"

echo "Resolving ${UPSTREAM_OWNER}/${UPSTREAM_REPO}@${target_ref} to a commit SHA..."

# Resolve via the GitHub API. Works for branch names, tag names, and full or
# abbreviated SHAs. Requires curl + python3.
resolve_sha() {
    local ref="$1"
    local api_url="https://api.github.com/repos/${UPSTREAM_OWNER}/${UPSTREAM_REPO}/commits/${ref}"
    local response
    if ! response=$(curl -sSfL -H "Accept: application/vnd.github+json" "$api_url"); then
        echo "ERROR: GitHub API request failed for ref '${ref}'." >&2
        echo "       Tried: ${api_url}" >&2
        exit 1
    fi
    python3 -c "import json,sys; print(json.loads(sys.argv[1])['sha'])" "$response"
}

new_sha="$(resolve_sha "$target_ref")"

if [[ -z "$new_sha" || ${#new_sha} -ne 40 ]]; then
    echo "ERROR: failed to resolve '${target_ref}' to a 40-char commit SHA." >&2
    exit 1
fi

echo "Pinning to ${UPSTREAM_OWNER}/${UPSTREAM_REPO}@${new_sha}"

# Rewrite every rev = "..." inside lines that also reference stacks-core.
# Python is used (rather than sed) so we never accidentally clobber a `rev`
# field that belongs to a different git dependency.
python3 - "$CARGO_TOML" "$UPSTREAM_URL" "$new_sha" <<'PY'
import re
import sys
from pathlib import Path

cargo_path = Path(sys.argv[1])
upstream_url = sys.argv[2]
new_sha = sys.argv[3]

text = cargo_path.read_text()
pattern = re.compile(
    r'(\{[^{}\n]*git\s*=\s*"' + re.escape(upstream_url) + r'"[^{}\n]*rev\s*=\s*")[0-9a-f]+("[^{}\n]*\})'
)
new_text, count = pattern.subn(lambda m: m.group(1) + new_sha + m.group(2), text)
if count == 0:
    print(f"WARNING: no rev = \"...\" pins found for {upstream_url} in {cargo_path}", file=sys.stderr)
cargo_path.write_text(new_text)
print(f"Rewrote {count} rev pin(s) in {cargo_path}")
PY

# Cargo doesn't pick up the new rev unless you explicitly tell it to refresh
# the package metadata. Pass --workspace so a transitive bump (e.g. a future
# crate added to MANAGED_PACKAGES) doesn't silently get missed.
echo "Refreshing Cargo.lock..."
update_args=()
for pkg in "${MANAGED_PACKAGES[@]}"; do
    update_args+=(-p "$pkg")
done
(cd "$REPO_ROOT" && cargo update "${update_args[@]}") || {
    echo
    echo "cargo update failed. Inspect the error above; the Cargo.toml pin was already updated." >&2
    exit 1
}

if [[ "${SKIP_CHECK:-0}" != "1" ]]; then
    echo "Running cargo check (set SKIP_CHECK=1 to skip)..."
    (cd "$REPO_ROOT" && cargo check --workspace --all-targets) || {
        echo
        echo "cargo check failed. The new pin may be incompatible with this crate;" >&2
        echo "you'll likely need to adjust the bindings code to match upstream changes." >&2
        exit 1
    }
fi

echo
echo "Pinned to https://github.com/${UPSTREAM_OWNER}/${UPSTREAM_REPO}/commit/${new_sha}"
