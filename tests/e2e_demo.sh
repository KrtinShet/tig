#!/bin/bash
# End-to-end demo: Two workspaces, different patches, test runs, review from passing snapshot, Git export

set -e

TIG="${TIG:-/Users/krtinshet/Development/tessera-vcs/target/debug/tig}"
DEMO_DIR="${DEMO_DIR:-/tmp/tig-demo}"

echo "=== Tig E2E Demo ==="
echo ""

# Clean up and init
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

echo "1. Initialize Tig project"
$TIG init --name demo-project

echo ""
echo "2. Create workspace A (Claude attempt)"
$TIG workspace create fix-bug --actor claude --goal "Fix the bug"

echo ""
echo "3. Write files in workspace A"
$TIG write /src/calc.js --content 'exports.add = function(a, b) { return a + b; };'
$TIG write /tests/calc.test.js --content 'const { add } = require("../src/calc.js"); if (add(2,3) !== 5) throw new Error("fail"); console.log("PASS");'

echo ""
echo "4. Run tests in workspace A"
$TIG run execute "node tests/calc.test.js" || true

echo ""
echo "5. Create workspace B (Codex attempt with different fix)"
$TIG workspace create fix-bug-alt --actor codex --goal "Fix the bug alt"
$TIG write /src/calc.js --content 'exports.add = function(a, b) { return a + b; };'
$TIG write /tests/calc.test.js --content 'const { add } = require("../src/calc.js"); if (add(2,3) !== 5) throw new Error("fail"); console.log("PASS");'

echo ""
echo "6. Run tests in workspace B"
$TIG run execute "node tests/calc.test.js" || true

echo ""
echo "7. List snapshots for workspace A"
$TIG workspace switch fix-bug
$TIG snapshot list

echo ""
echo "8. Create review unit from latest snapshot"
# Get the first snapshot ID as target (base snapshot)
FIRST_SNAPSHOT=$($TIG snapshot list 2>/dev/null | head -2 | tail -1 | awk '{print $1}')
$TIG review create --from latest --target "$FIRST_SNAPSHOT"

echo ""
echo "9. List review units"
$TIG review list

echo ""
echo "10. Export review to Git"
REVIEW_ID=$($TIG review list 2>/dev/null | tail -1 | awk '{print $1}')
if [ -n "$REVIEW_ID" ]; then
    $TIG git export --review "$REVIEW_ID"
fi

echo ""
echo "=== Demo Complete ==="
