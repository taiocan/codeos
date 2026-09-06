#!/usr/bin/env bash
# Render the fixed `mechanics:` block of a downstream codeos.yaml from a Codeos Mechanics policy.
# The policy's grouped tables are the single source of the fixed set; this emits the transparency
# view a project's codeos.yaml carries. Used by the initializer to emit the exact set for the
# selected policy, and by project-config-contract.sh to validate an existing block against it.
#
# Usage: render-mechanics-block.sh <path-to-codeos-mechanics/vN.md>
set -euo pipefail

POLICY="${1:?usage: render-mechanics-block.sh <codeos-mechanics policy .md>}"
[[ -f "${POLICY}" ]] || { printf 'render-mechanics-block: no such policy: %s\n' "${POLICY}" >&2; exit 1; }

awk '
  BEGIN {
    print "mechanics:"
    print "  # Fixed by the selected Codeos Mechanics policy. Displayed for transparency only; a"
    print "  # project cannot change one. project-config-contract.sh fails closed on any deviation."
  }
  /^### Delivery$/      { print "  delivery:";      next }
  /^### Validation$/    { print "  validation:";    next }
  /^### Communication$/ { print "  communication:"; next }
  /^\| `[a-z_]+` \| (always|always_when_gui|always_when_persistence) \|/ {
    line = $0
    sub(/^\| `/, "", line)
    name = line; sub(/`.*/, "", name)
    rest = line; sub(/^[a-z_]+` \| /, "", rest)
    applies = rest; sub(/ \|.*/, "", applies)
    printf "    %s: %s\n", name, applies
  }
' "${POLICY}"
