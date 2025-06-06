#!/bin/bash

# Set locale for reproducibility
export LC_ALL=C
export LANG=C

# Ensure script will fail correctly.
set -o errexit -o nounset -o pipefail

# Check that the required tools are available
command -v cargo >/dev/null 2>&1 || { echo >&2 "cargo is required but it's not installed. Aborting."; exit 1; }
command -v grep >/dev/null 2>&1 || { echo >&2 "grep is required but it's not installed. Aborting."; exit 1; }
command -v sed >/dev/null 2>&1 || { echo >&2 "sed is required but it's not installed. Aborting."; exit 1; }
command -v bc >/dev/null 2>&1 || { echo >&2 "bc is required but it's not installed. Aborting."; exit 1; }

# Directory setup
ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$ROOT_DIR"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=======================================================${NC}"
echo -e "${BLUE}         Minilate Benchmarking Script                  ${NC}"
echo -e "${BLUE}=======================================================${NC}"

# Cleanup any old build files
echo -e "\n${GREEN}Cleaning up old build files...${NC}"
cargo clean

# Build all benchmarks with release optimizations
echo -e "\n${GREEN}Building benchmarks with release optimizations...${NC}"
cargo build --profile bench --bench bench_minilate
cargo build --profile bench --bench bench_handlebars
cargo build --profile bench --bench bench_minijinja

# Directory where binaries are located
TARGET_DIR="$ROOT_DIR/target/release/deps"

# Function to get binary size
get_binary_size() {
    local binary_path="$1"
    if [[ -f "$binary_path" ]]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            stat -f%z "$binary_path"
        else
            stat -c%s "$binary_path"
        fi
    else
        echo "0"
    fi
}

# Format file size in human-readable format
format_size() {
    local size_bytes=$1

    if ((size_bytes >= 1048576)); then
        echo "$(bc <<< "scale=2; $size_bytes / 1048576") MB"
    elif ((size_bytes >= 1024)); then
        echo "$(bc <<< "scale=2; $size_bytes / 1024") KB"
    else
        echo "$size_bytes bytes"
    fi
}

echo -e "\n${GREEN}Analyzing binary sizes...${NC}"

# Find the benchmark binaries
MINILATE_BIN=$(find "$TARGET_DIR" -name "bench_minilate-*" -not -name "*.d" | sort | head -n 1)
HANDLEBARS_BIN=$(find "$TARGET_DIR" -name "bench_handlebars-*" -not -name "*.d" | sort | head -n 1)
MINIJINJA_BIN=$(find "$TARGET_DIR" -name "bench_minijinja-*" -not -name "*.d" | sort | head -n 1)

# Get sizes
MINILATE_SIZE=$(get_binary_size "$MINILATE_BIN")
HANDLEBARS_SIZE=$(get_binary_size "$HANDLEBARS_BIN")
MINIJINJA_SIZE=$(get_binary_size "$MINIJINJA_BIN")

echo -e "${YELLOW}Binary Sizes:${NC}"
echo -e "Minilate:   $(format_size "$MINILATE_SIZE")"
echo -e "Handlebars: $(format_size "$HANDLEBARS_SIZE")"
echo -e "MiniJinja:  $(format_size "$MINIJINJA_SIZE")"

echo -e "\n${GREEN}Running benchmarks...${NC}"
echo -e "${YELLOW}This may take a few minutes. Each benchmark runs 100 templates 50 times.${NC}\n"

# Create temp files to capture benchmark results
MINILATE_RESULT=$(mktemp)
HANDLEBARS_RESULT=$(mktemp)
MINIJINJA_RESULT=$(mktemp)

# Cleanup on exit
cleanup() {
    rm -f "$MINILATE_RESULT" "$HANDLEBARS_RESULT" "$MINIJINJA_RESULT"
}
trap cleanup EXIT

# Run the benchmarks and capture outputs
echo -e "${BLUE}----- Minilate Benchmark -----${NC}"
cargo bench --bench bench_minilate | tee "$MINILATE_RESULT" || true

echo -e "\n${BLUE}----- Handlebars Benchmark -----${NC}"
cargo bench --bench bench_handlebars | tee "$HANDLEBARS_RESULT" || true

echo -e "\n${BLUE}----- MiniJinja Benchmark -----${NC}"
cargo bench --bench bench_minijinja | tee "$MINIJINJA_RESULT" || true

# Function to extract benchmark time from output
extract_time() {
    local file="$1"
    # Look for lines like: time:   [140.79 µs 141.00 µs 141.16 µs]
    # Extract the middle value (average)
    if grep -q "time:" "$file"; then
        local time_line=$(grep "time:" "$file" | head -n 1)
        if [[ "$time_line" =~ \[([0-9.]+)\ µs\ ([0-9.]+)\ µs ]]; then
            echo "${BASH_REMATCH[2]}"
        else
            echo "0"
        fi
    else
        echo "0"
    fi
}

# Extract benchmark times
MINILATE_TIME=$(extract_time "$MINILATE_RESULT")
HANDLEBARS_TIME=$(extract_time "$HANDLEBARS_RESULT")
MINIJINJA_TIME=$(extract_time "$MINIJINJA_RESULT")

# Calculate relative performance and size (using Minilate as baseline)
if [[ "$MINILATE_TIME" != "0" && "$HANDLEBARS_TIME" != "0" && "$MINIJINJA_TIME" != "0" ]]; then
    MINILATE_RELATIVE="1.00x"
    HANDLEBARS_RELATIVE=$(printf "%.2fx" "$(echo "scale=4; $HANDLEBARS_TIME / $MINILATE_TIME" | bc)")
    MINIJINJA_RELATIVE=$(printf "%.2fx" "$(echo "scale=4; $MINIJINJA_TIME / $MINILATE_TIME" | bc)")
else
    MINILATE_RELATIVE="N/A"
    HANDLEBARS_RELATIVE="N/A"
    MINIJINJA_RELATIVE="N/A"
fi

# Calculate relative size (using Minilate as baseline)
if [[ "$MINILATE_SIZE" != "0" && "$HANDLEBARS_SIZE" != "0" && "$MINIJINJA_SIZE" != "0" ]]; then
    MINILATE_SIZE_RELATIVE="1.00x"
    HANDLEBARS_SIZE_RELATIVE=$(printf "%.2fx" "$(echo "scale=4; $HANDLEBARS_SIZE / $MINILATE_SIZE" | bc)")
    MINIJINJA_SIZE_RELATIVE=$(printf "%.2fx" "$(echo "scale=4; $MINIJINJA_SIZE / $MINILATE_SIZE" | bc)")
else
    MINILATE_SIZE_RELATIVE="N/A"
    HANDLEBARS_SIZE_RELATIVE="N/A"
    MINIJINJA_SIZE_RELATIVE="N/A"
fi

# Format times for display
if [[ "$MINILATE_TIME" != "0" ]]; then
    MINILATE_TIME_DISPLAY="${MINILATE_TIME} µs"
else
    MINILATE_TIME_DISPLAY="N/A"
fi

if [[ "$HANDLEBARS_TIME" != "0" ]]; then
    HANDLEBARS_TIME_DISPLAY="${HANDLEBARS_TIME} µs"
else
    HANDLEBARS_TIME_DISPLAY="N/A"
fi

if [[ "$MINIJINJA_TIME" != "0" ]]; then
    MINIJINJA_TIME_DISPLAY="${MINIJINJA_TIME} µs"
else
    MINIJINJA_TIME_DISPLAY="N/A"
fi

echo -e "\n${GREEN}All benchmarks completed!${NC}"

# Create a nice ASCII table with the results
echo -e "\n${BLUE}=======================================================${NC}"
echo -e "${BLUE}                 Benchmark Results                      ${NC}"
echo -e "${BLUE}=======================================================${NC}\n"

# Function to print a horizontal line
print_line() {
    printf "+"
    printf "%0.s-" {1..12}
    printf "+"
    printf "%0.s-" {1..17}
    printf "+"
    printf "%0.s-" {1..17}
    printf "+"
    printf "%0.s-" {1..17}
    printf "+"
    printf "%0.s-" {1..17}
    printf "+\n"
}

# Print table header
print_line
printf "| %-10s | %-15s | %-15s | %-15s | %-15s |\n" "Engine" "Binary Size" "Relative Size" "Time/Template" "Relative Perf."
print_line

# Print table rows
printf "| %-10s | %-15s | %-15s | %-15s  | %-15s |\n" "Minilate" "$(format_size "$MINILATE_SIZE")" "$MINILATE_SIZE_RELATIVE" "${MINILATE_TIME_DISPLAY}" "$MINILATE_RELATIVE"
printf "| %-10s | %-15s | %-15s | %-15s  | %-15s |\n" "Handlebars" "$(format_size "$HANDLEBARS_SIZE")" "$HANDLEBARS_SIZE_RELATIVE" "${HANDLEBARS_TIME_DISPLAY}" "$HANDLEBARS_RELATIVE"
printf "| %-10s | %-15s | %-15s | %-15s  | %-15s |\n" "MiniJinja" "$(format_size "$MINIJINJA_SIZE")" "$MINIJINJA_SIZE_RELATIVE" "${MINIJINJA_TIME_DISPLAY}" "$MINIJINJA_RELATIVE"

# Print table footer
print_line

echo -e "\n${YELLOW}Note: Relative Performance and Relative Size use Minilate as the baseline (1.00x).${NC}"
echo -e "${YELLOW}Lower times and smaller binary sizes are better.${NC}"
echo -e "${YELLOW}Higher relative performance numbers mean slower execution compared to Minilate.${NC}"
echo -e "${YELLOW}Higher relative size numbers mean larger binaries compared to Minilate.${NC}"
echo -e "${YELLOW}HTML reports with detailed metrics are available in the target/criterion directory.${NC}"

exit 0
