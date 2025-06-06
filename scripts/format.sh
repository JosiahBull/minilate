#!/bin/bash

# Set locale for reproducibility
export LC_ALL=C
export LANG=C

# Ensure script will fail correctly.
set -o errexit -o nounset -o pipefail

# Directory setup
ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$ROOT_DIR"

# Colors for output
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}         Minilate Format Script                         ${NC}"
echo -e "${BLUE}=======================================================${NC}"

# Simply call the lint script with the --fix flag
echo -e "Running linting in fix mode..."
"$ROOT_DIR/scripts/lint.sh" --fix

exit 0