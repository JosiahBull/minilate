#!/bin/bash

# Set locale for reproducibility
export LC_ALL=C
export LANG=C

# Ensure script will fail correctly.
set -o errexit -o nounset -o pipefail

# Check that the required tools are available
command -v cargo >/dev/null 2>&1 || { echo >&2 "cargo is required but it's not installed. Aborting."; exit 1; }

# Directory setup
ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$ROOT_DIR"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}         Minilate Test Script                          ${NC}"
echo -e "${BLUE}=======================================================${NC}"

# Function to handle errors
handle_error() {
    echo -e "\n${RED}Tests failed!${NC}"
    exit 1
}

# Set trap for error handling
trap handle_error ERR

echo -e "\n${GREEN}Running Minilate test suite...${NC}"
cargo test --all-features --all-targets

echo -e "\n${GREEN}All tests passed successfully!${NC}"
echo -e "${YELLOW}For more detailed output, run:${NC}"
echo -e "${YELLOW}  cargo test -- --nocapture${NC}"

exit 0
