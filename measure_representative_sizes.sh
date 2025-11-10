#!/bin/bash
# Measure binary sizes for a representative benchmark (fibonacci) across all profiles

BENCHMARK="fibonacci"
OUTPUT_FILE="binary_sizes_fibonacci.json"

echo "Measuring binary sizes for $BENCHMARK across all pathfinder profiles..."
echo ""

# Profiles to measure (from pathfinder study)
PROFILES=(
    "baseline"
    "standard-release"
    "lto-thin"
    "lto-fat"
    "codegen-1"
    "cpu-native"
    "opt-s"
    "size-s-lto"
    "size-s-strip"
    "size-z-lto"
    "size-z-strip"
    "size-ultra"
    "size-s-native"
    "size-z-native"
    "perf-ultra"
)

# Start JSON output
echo "{" > "$OUTPUT_FILE"
echo "  \"benchmark\": \"$BENCHMARK\"," >> "$OUTPUT_FILE"
echo "  \"sizes\": {" >> "$OUTPUT_FILE"

FIRST=true

for PROFILE in "${PROFILES[@]}"; do
    echo "Building $BENCHMARK with $PROFILE..."

    # Build the binary
    cargo build -p "$BENCHMARK" --profile "$PROFILE" --quiet 2>/dev/null

    # Find the binary
    BINARY_PATH=$(find target -name "$BENCHMARK" -type f -path "*/$PROFILE/*" 2>/dev/null | head -1)

    if [ -z "$BINARY_PATH" ]; then
        # Try alternative paths
        BINARY_PATH=$(find target -name "$BENCHMARK" -type f 2>/dev/null | head -1)
    fi

    if [ -n "$BINARY_PATH" ] && [ -f "$BINARY_PATH" ]; then
        SIZE=$(stat -c%s "$BINARY_PATH" 2>/dev/null || stat -f%z "$BINARY_PATH" 2>/dev/null)
        SIZE_KB=$(echo "scale=2; $SIZE / 1024" | bc)
        SIZE_MB=$(echo "scale=3; $SIZE / 1024 / 1024" | bc)

        echo "  ✓ $PROFILE: ${SIZE_KB} KB"

        # Add JSON entry
        if [ "$FIRST" = false ]; then
            echo "," >> "$OUTPUT_FILE"
        fi
        FIRST=false

        echo -n "    \"$PROFILE\": {\"bytes\": $SIZE, \"kb\": $SIZE_KB, \"mb\": $SIZE_MB}" >> "$OUTPUT_FILE"
    else
        echo "  ✗ $PROFILE: Binary not found"
    fi
done

# Close JSON
echo "" >> "$OUTPUT_FILE"
echo "  }" >> "$OUTPUT_FILE"
echo "}" >> "$OUTPUT_FILE"

echo ""
echo "✅ Binary sizes saved to $OUTPUT_FILE"

# Display summary
echo ""
echo "Summary:"
python3 -c "
import json
with open('$OUTPUT_FILE') as f:
    data = json.load(f)
sizes = [(p, s['kb']) for p, s in data['sizes'].items()]
sizes.sort(key=lambda x: x[1])
print(f\"Smallest: {sizes[0][0]} ({sizes[0][1]} KB)\")
print(f\"Largest: {sizes[-1][0]} ({sizes[-1][1]} KB)\")
ratio = sizes[-1][1] / sizes[0][1]
print(f\"Ratio: {ratio:.1f}x\")
" 2>/dev/null || echo "Install python3 to see summary"
