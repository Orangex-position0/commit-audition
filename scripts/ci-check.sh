#!/usr/bin/env bash
# 本地 CI 等价检查脚本
# 用法: ./scripts/ci-check.sh

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass=0
fail=0

run_check() {
    local name="$1"
    shift
    echo -e "${YELLOW}[CHECK]${NC} $name"
    if "$@"; then
        echo -e "${GREEN}[PASS]${NC} $name"
        ((pass++))
    else
        echo -e "${RED}[FAIL]${NC} $name"
        ((fail++))
    fi
}

echo "========== CI Local Check =========="
echo ""

run_check "cargo check --locked" cargo check --locked
run_check "cargo fmt --check"     cargo fmt --check
run_check "cargo clippy"          cargo clippy -- -D warnings
run_check "cargo test"            cargo test

echo ""
echo "========== Results =========="
echo -e "  ${GREEN}Pass: ${pass}${NC}"
echo -e "  ${RED}Fail: ${fail}${NC}"

if ((fail > 0)); then
    exit 1
fi
