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
command -v jq >/dev/null 2>&1 || { echo >&2 "jq is required but it's not installed. Aborting."; exit 1; }

# Directory setup
ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
RESULTS_DIR="$ROOT_DIR/benchmark_results"
cd "$ROOT_DIR"

# Get current git information
CURRENT_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
CURRENT_COMMIT_SHORT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
PREVIOUS_COMMIT=$(git rev-parse HEAD~1 2>/dev/null || echo "unknown")
PREVIOUS_COMMIT_SHORT=$(git rev-parse --short HEAD~1 2>/dev/null || echo "unknown")
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Check if git workspace is dirty
GIT_DIRTY=""
if git rev-parse --git-dir >/dev/null 2>&1; then
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        GIT_DIRTY="true"
    fi
fi

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

# Function to find the most recent benchmark results file
find_most_recent_results() {
    local exclude_commit="$1"
    local most_recent=""
    local most_recent_time=0

    for results_file in "$RESULTS_DIR"/benchmark_*.json; do
        if [[ -f "$results_file" ]]; then
            # Extract commit hash from filename
            local file_commit=$(basename "$results_file" | sed 's/benchmark_\(.*\)\.json/\1/')

            # Skip if this is the current commit
            if [[ "$file_commit" == "$exclude_commit" ]]; then
                continue
            fi

            # Get file modification time
            if [[ "$OSTYPE" == "darwin"* ]]; then
                local file_time=$(stat -f%m "$results_file")
            else
                local file_time=$(stat -c%Y "$results_file")
            fi

            if [[ "$file_time" -gt "$most_recent_time" ]]; then
                most_recent_time="$file_time"
                most_recent="$results_file"
            fi
        fi
    done

    echo "$most_recent"
}

# Function to load benchmark results for a specific commit
load_benchmark_results() {
    local commit_hash="$1"
    local results_file="$RESULTS_DIR/benchmark_${commit_hash}.json"

    if [[ -f "$results_file" ]]; then
        echo "$results_file"
    else
        echo ""
    fi
}

# Function to save benchmark results
save_benchmark_results() {
    local commit_hash="$1"
    local minilate_time="$2"
    local handlebars_time="$3"
    local minijinja_time="$4"
    local minilate_size="$5"
    local handlebars_size="$6"
    local minijinja_size="$7"

    local results_file="$RESULTS_DIR/benchmark_${commit_hash}.json"

    cat > "$results_file" << EOF
{
  "timestamp": "$TIMESTAMP",
  "commit": "$commit_hash",
  "commit_short": "$(git rev-parse --short "$commit_hash" 2>/dev/null || echo "unknown")",
  "branch": "$CURRENT_BRANCH",
  "results": {
    "minilate": {
      "time_us": "$minilate_time",
      "binary_size_bytes": "$minilate_size"
    },
    "handlebars": {
      "time_us": "$handlebars_time",
      "binary_size_bytes": "$handlebars_size"
    },
    "minijinja": {
      "time_us": "$minijinja_time",
      "binary_size_bytes": "$minijinja_size"
    }
  }
}
EOF
    echo "Benchmark results saved to: $results_file"
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

# Save current benchmark results only if workspace is clean
if [[ "$GIT_DIRTY" == "true" ]]; then
    echo -e "\n${YELLOW}⚠️  Git workspace is dirty - benchmark results will NOT be saved!${NC}"
else
    save_benchmark_results "$CURRENT_COMMIT" "$MINILATE_TIME" "$HANDLEBARS_TIME" "$MINIJINJA_TIME" "$MINILATE_SIZE" "$HANDLEBARS_SIZE" "$MINIJINJA_SIZE"
fi

# Load previous results for comparison
LAST_RESULTS_FILE=$(find_most_recent_results "$CURRENT_COMMIT")
PREVIOUS_RESULTS_FILE=$(load_benchmark_results "$PREVIOUS_COMMIT")

# Extract previous benchmark data
echo -e "\n${GREEN}Loading historical benchmark data...${NC}"
if [[ -n "$LAST_RESULTS_FILE" && -f "$LAST_RESULTS_FILE" ]]; then
    echo -e "${YELLOW}Found most recent results: $(basename "$LAST_RESULTS_FILE")${NC}"
    LAST_MINILATE_TIME=$(jq -r ".results.minilate.time_us // \"N/A\"" "$LAST_RESULTS_FILE" 2>/dev/null || echo "N/A")
    LAST_HANDLEBARS_TIME=$(jq -r ".results.handlebars.time_us // \"N/A\"" "$LAST_RESULTS_FILE" 2>/dev/null || echo "N/A")
    LAST_MINIJINJA_TIME=$(jq -r ".results.minijinja.time_us // \"N/A\"" "$LAST_RESULTS_FILE" 2>/dev/null || echo "N/A")
else
    echo -e "${YELLOW}No previous benchmark results found${NC}"
    LAST_MINILATE_TIME="N/A"
    LAST_HANDLEBARS_TIME="N/A"
    LAST_MINIJINJA_TIME="N/A"
fi

if [[ -n "$PREVIOUS_RESULTS_FILE" && -f "$PREVIOUS_RESULTS_FILE" ]]; then
    echo -e "${YELLOW}Found previous commit results: $(basename "$PREVIOUS_RESULTS_FILE")${NC}"
    PREV_MINILATE_TIME=$(jq -r ".results.minilate.time_us // \"N/A\"" "$PREVIOUS_RESULTS_FILE" 2>/dev/null || echo "N/A")
    PREV_HANDLEBARS_TIME=$(jq -r ".results.handlebars.time_us // \"N/A\"" "$PREVIOUS_RESULTS_FILE" 2>/dev/null || echo "N/A")
    PREV_MINIJINJA_TIME=$(jq -r ".results.minijinja.time_us // \"N/A\"" "$PREVIOUS_RESULTS_FILE" 2>/dev/null || echo "N/A")
else
    echo -e "${YELLOW}No benchmark results found for previous commit (${PREVIOUS_COMMIT_SHORT})${NC}"
    PREV_MINILATE_TIME="N/A"
    PREV_HANDLEBARS_TIME="N/A"
    PREV_MINIJINJA_TIME="N/A"
fi

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
    printf "%0.s-" {1..15}
    printf "+"
    printf "%0.s-" {1..13}
    printf "+"
    printf "%0.s-" {1..15}
    printf "+"
    printf "%0.s-" {1..13}
    printf "+"
    printf "%0.s-" {1..11}
    printf "+"
    printf "%0.s-" {1..13}
    printf "+\n"
}

# Print current git info
echo -e "${YELLOW}Current commit: ${CURRENT_COMMIT_SHORT} (${CURRENT_BRANCH})${NC}"
echo -e "${YELLOW}Previous commit: ${PREVIOUS_COMMIT_SHORT}${NC}"

# Print table header
print_line
printf "| %-10s | %-13s | %-11s | %-13s | %-11s | %-9s | %-11s |\n" "Engine" "Binary Size" "Rel. Size" "Time/Template" "Rel. Perf." "Last Run" "Prev Commit"
print_line

# Format historical times for display
format_time_display() {
    local time="$1"
    if [[ "$time" != "N/A" && "$time" != "0" ]]; then
        echo "${time} µs"
    else
        echo "N/A"
    fi
}

# Print table rows
printf "| %-10s | %-13s | %-11s | %-13s  | %-11s | %-9s | %-11s |\n" \
    "Minilate" "$(format_size "$MINILATE_SIZE")" "$MINILATE_SIZE_RELATIVE" "${MINILATE_TIME_DISPLAY}" "$MINILATE_RELATIVE" \
    "$(format_time_display "$LAST_MINILATE_TIME")" "$(format_time_display "$PREV_MINILATE_TIME")"

printf "| %-10s | %-13s | %-11s | %-13s | %-11s | %-9s | %-11s |\n" \
    "Handlebars" "$(format_size "$HANDLEBARS_SIZE")" "$HANDLEBARS_SIZE_RELATIVE" "${HANDLEBARS_TIME_DISPLAY}" "$HANDLEBARS_RELATIVE" \
    "$(format_time_display "$LAST_HANDLEBARS_TIME")" "$(format_time_display "$PREV_HANDLEBARS_TIME")"

printf "| %-10s | %-13s | %-11s | %-13s | %-11s | %-9s | %-11s |\n" \
    "MiniJinja" "$(format_size "$MINIJINJA_SIZE")" "$MINIJINJA_SIZE_RELATIVE" "${MINIJINJA_TIME_DISPLAY}" "$MINIJINJA_RELATIVE" \
    "$(format_time_display "$LAST_MINIJINJA_TIME")" "$(format_time_display "$PREV_MINIJINJA_TIME")"

# Print table footer
print_line

echo -e "\n${YELLOW}Note: Relative Performance and Relative Size use Minilate as the baseline (1.00x).${NC}"
echo -e "${YELLOW}Prev Commit: Benchmark results from the previous commit (${PREVIOUS_COMMIT_SHORT}).${NC}"

exit 0
