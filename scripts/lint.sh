#!/bin/bash

# Set locale for reproducibility
export LC_ALL=C
export LANG=C

# Ensure script will fail correctly.
set -o errexit -o nounset -o pipefail

# Check that the required tools are available
command -v cargo >/dev/null 2>&1 || { echo >&2 "cargo is required but it's not installed. Aborting."; exit 1; }
command -v shellcheck >/dev/null 2>&1 || { echo >&2 "shellcheck is required but it's not installed. Aborting."; exit 1; }
if ! rustup toolchain list | grep -q 'nightly'; then
    echo "Nightly Rust toolchain is required but not installed. Please run:"
    echo "  rustup toolchain install nightly"
    exit 1
fi

# Directory setup
ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$ROOT_DIR"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default to check mode
FIX_MODE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --fix)
            FIX_MODE=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Usage: $0 [--fix]"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}         Minilate Linting Script                       ${NC}"
echo -e "${BLUE}=======================================================${NC}"

# Function to handle errors
handle_error() {
    echo -e "\n${RED}Linting failed!${NC}"
    exit 1
}

# Set trap for error handling
trap handle_error ERR

if [ "$FIX_MODE" = true ]; then
    echo -e "\n${GREEN}Running cargo fmt to format code...${NC}"
    cargo +nightly fmt

    echo -e "\n${GREEN}Running cargo clippy with fixes...${NC}"
    cargo clippy --all-targets --all-features --fix --allow-dirty

    echo -e "\n${GREEN}Running shellcheck on shell scripts...${NC}"
    find . -name "*.sh" -type f -exec shellcheck {} +

    echo -e "\n${GREEN}Code formatting and linting completed successfully!${NC}"
else
    echo -e "\n${GREEN}Checking code formatting with cargo fmt...${NC}"
    cargo fmt -- --check

    echo -e "\n${GREEN}Checking code with cargo clippy...${NC}"
    cargo clippy --all-targets --all-features -- -D warnings

    echo -e "\n${GREEN}Running shellcheck on shell scripts...${NC}"
    find . -name "*.sh" -type f -exec shellcheck {} +

    echo -e "\n${GREEN}Code formatting and linting checks passed successfully!${NC}"
    echo -e "${YELLOW}To automatically fix issues, run with the --fix flag:${NC}"
    echo -e "${YELLOW}  ./scripts/lint.sh --fix${NC}"
fi

exit 0
