#!/usr/bin/env bash
# pre-push hook: push 前自动运行 CI 等价检查
# 安装方式: cp scripts/pre-push.sh .git/hooks/pre-push

set -euo pipefail

echo "Running CI checks before push..."

if ! ./scripts/ci-check.sh; then
    echo ""
    echo "Push aborted: CI checks failed. Fix the issues above or use --no-verify to skip."
    exit 1
fi

exit 0
