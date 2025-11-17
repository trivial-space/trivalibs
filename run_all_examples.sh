#!/bin/bash

# Script to run all Cargo examples sequentially
# Each example runs until closed, then the next one starts

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
RELEASE_MODE=false
DRY_RUN=false
CONTINUE_ON_ERROR=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --stop-on-error)
            CONTINUE_ON_ERROR=false
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --release          Build and run examples in release mode"
            echo "  --dry-run          Show which examples would be run without executing"
            echo "  --stop-on-error    Stop if any example fails (default: continue)"
            echo "  -h, --help         Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Function to get list of examples
get_examples() {
    # Extract example names from Cargo.toml
    # Look for [[example]] sections and extract the name field
    local examples=()

    # Use cargo metadata to get examples
    if command -v jq &> /dev/null; then
        # If jq is available, use it for cleaner parsing
        examples=($(cargo metadata --no-deps --format-version 1 2>/dev/null | \
            jq -r '.packages[] | select(.name == "trivalibs_examples") | .targets[] | select(.kind[] | contains("example")) | .name' | \
            sort))
    else
        # Fallback: parse the examples directory
        if [ -d "examples" ]; then
            for dir in examples/*/; do
                if [ -d "$dir" ]; then
                    example_name=$(basename "$dir")
                    # Skip if it's the triangle example (it has its own structure)
                    if [ -f "examples/$example_name/main.rs" ]; then
                        examples+=("$example_name")
                    fi
                fi
            done
            # Add triangle separately if it exists
            if [ -d "examples/triangle/src" ]; then
                examples+=("simple_triangle")
            fi
            # Sort the array
            IFS=$'\n' examples=($(sort <<<"${examples[*]}"))
            unset IFS
        fi
    fi

    echo "${examples[@]}"
}

# Get all examples
EXAMPLES=($(get_examples))
TOTAL=${#EXAMPLES[@]}

if [ $TOTAL -eq 0 ]; then
    echo -e "${RED}Error: No examples found${NC}"
    exit 1
fi

echo -e "${BLUE}Found $TOTAL examples${NC}"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}Dry run mode - would execute:${NC}"
    for i in "${!EXAMPLES[@]}"; do
        num=$((i + 1))
        echo "  $num. ${EXAMPLES[$i]}"
    done
    exit 0
fi

# Build command
if [ "$RELEASE_MODE" = true ]; then
    BUILD_FLAG="--release"
    echo -e "${YELLOW}Running in release mode${NC}"
else
    BUILD_FLAG=""
    echo -e "${YELLOW}Running in debug mode${NC}"
fi
echo ""

# Counter for successful runs
SUCCESS_COUNT=0
FAILED_COUNT=0
SKIPPED_COUNT=0
FAILED_EXAMPLES=()

# Function to run a single example
run_example() {
    local example_name=$1
    local index=$2

    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}Running example $index/$TOTAL: $example_name${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Run the example
    if cargo run $BUILD_FLAG --example "$example_name"; then
        echo ""
        echo -e "${GREEN}✓ Example '$example_name' completed successfully${NC}"
        ((SUCCESS_COUNT++))
    else
        exit_code=$?
        echo ""
        echo -e "${RED}✗ Example '$example_name' failed with exit code $exit_code${NC}"
        ((FAILED_COUNT++))
        FAILED_EXAMPLES+=("$example_name")

        if [ "$CONTINUE_ON_ERROR" = false ]; then
            echo -e "${RED}Stopping due to error (--stop-on-error is set)${NC}"
            return 1
        else
            echo -e "${YELLOW}Continuing to next example...${NC}"
        fi
    fi

    echo ""
    return 0
}

# Main loop
for i in "${!EXAMPLES[@]}"; do
    example="${EXAMPLES[$i]}"
    index=$((i + 1))

    if ! run_example "$example" "$index"; then
        break
    fi

    # Add a small pause between examples
    if [ $index -lt $TOTAL ]; then
        sleep 0.5
    fi
done

# Summary
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Successful: $SUCCESS_COUNT${NC}"
if [ $FAILED_COUNT -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED_COUNT${NC}"
    echo -e "${RED}Failed examples:${NC}"
    for failed in "${FAILED_EXAMPLES[@]}"; do
        echo -e "${RED}  - $failed${NC}"
    done
fi
echo -e "${BLUE}Total: $TOTAL${NC}"
echo ""

if [ $FAILED_COUNT -eq 0 ]; then
    echo -e "${GREEN}All examples completed successfully! 🎉${NC}"
    exit 0
else
    exit 1
fi
