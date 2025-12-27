#!/bin/bash
# Test runner for Sweepr using the dependencies fixture

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FIXTURE_DIR="$PROJECT_ROOT/tests/fixtures/dependencies"
SWEEPR_BIN="$PROJECT_ROOT/target/debug/sweepr"

echo "🧪 Testing Sweepr with dependencies fixture"
echo "============================================"
echo ""

# Build the binary first
echo "📦 Building Sweepr..."
cd "$PROJECT_ROOT"
cargo build --quiet

echo ""
echo "📂 Fixture: $FIXTURE_DIR"
echo ""

cd "$FIXTURE_DIR"

echo "🔍 Running analysis..."
echo ""

# Run sweepr
"$SWEEPR_BIN" check --entry entry.ts

echo ""
echo "✅ Test completed"
echo ""
echo "Expected results:"
echo "  - Unused dependencies: fs-extra, mocha, stream"
echo "  - Used dependencies: @sindresorhus/is, has, JSONStream, @tootallnate/once"
echo "  - Unused files: unused-module.ts"
echo "  - Used files: entry.ts, my-module.ts"
