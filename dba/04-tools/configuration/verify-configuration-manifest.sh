#!/usr/bin/env bash
# Regression guard against accidental in-place edits to approved DBA compositions.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
cd "${ROOT}"
sha256sum --check dba/00-entry/configurations/SHA256SUMS
