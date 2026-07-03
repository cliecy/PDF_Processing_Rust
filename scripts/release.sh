#!/usr/bin/env bash
#
# 发版脚本：把 main 上已提交的改动发布出去，触发 GitHub Actions 构建。
#
# 流程：同步远程 -> (可选)改版本号 -> 本地测试+构建 -> 推送 main
#       -> 把 main 合并进 release -> 推送 release（触发 publish workflow）
#
# 用法：
#   ./scripts/release.sh            # 自动递增 patch 版本号（如 1.0.1 -> 1.0.2）再发版
#   ./scripts/release.sh 1.1.0      # 指定新版本号发版
#   SKIP_CHECKS=1 ./scripts/release.sh   # 跳过本地 cargo test / npm run build
#
# 版本号必须是新的：如果对应 tag（app-vX.Y.Z）已存在于远程，脚本会拒绝执行，
# 避免覆盖已发布 release 的产物。
#
# 前提：改动已经提交到 main（或当前分支干净）；已安装并登录 gh CLI。

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# ---------- 推送方式：优先 gh 凭据走 HTTPS（本机 SSH 未配置 known_hosts） ----------
origin_url=$(git remote get-url origin)
if [[ $origin_url =~ ^git@github.com:(.+)$ ]]; then
    https_url="https://github.com/${BASH_REMATCH[1]}"
else
    https_url=$origin_url
fi

if command -v gh >/dev/null && gh auth status >/dev/null 2>&1; then
    git_remote() { git -c 'credential.helper=!gh auth git-credential' "$1" "$https_url" "${@:2}"; }
else
    git_remote() { git "$1" origin "${@:2}"; }
fi

# ---------- 前置检查 ----------
if [[ -n "$(git status --porcelain --untracked-files=no)" ]]; then
    echo "错误：工作区有未提交的改动，请先提交或 stash。" >&2
    exit 1
fi

orig_branch=$(git rev-parse --abbrev-ref HEAD)
trap 'git checkout -q "$orig_branch" 2>/dev/null || true' EXIT

echo "==> 同步远程分支"
git_remote fetch main:refs/remotes/origin/main release:refs/remotes/origin/release

git checkout -q main
git rebase --quiet origin/main

# ---------- 确定新版本号（默认递增 patch） ----------
current_version=$(node -p "require('./src-tauri/tauri.conf.json').version")
if [[ $# -ge 1 ]]; then
    version=$1
else
    version=$(echo "$current_version" | awk -F. '{printf "%d.%d.%d", $1, $2, $3 + 1}')
fi

if [[ $version == "$current_version" ]]; then
    echo "错误：新版本号与当前版本（$current_version）相同，发版必须递增版本号。" >&2
    exit 1
fi

# 防覆盖保险：目标 tag 已存在于远程则中止
if [[ -n "$(git_remote ls-remote "refs/tags/app-v$version")" ]]; then
    echo "错误：tag app-v$version 已存在于远程，继续会覆盖该版本的 release 产物。" >&2
    exit 1
fi

echo "==> 更新版本号：$current_version -> $version"
npm version --no-git-tag-version "$version" >/dev/null
node -e "
    const fs = require('fs');
    const p = 'src-tauri/tauri.conf.json';
    const conf = JSON.parse(fs.readFileSync(p, 'utf8'));
    conf.version = '$version';
    fs.writeFileSync(p, JSON.stringify(conf, null, 2) + '\n');
"
# 只替换第一个行首的 version =（即 [package] 的版本，依赖里的 version 不在行首）
VERSION="$version" perl -0777 -pi -e \
    's/^version = "[^"]*"/version = "$ENV{VERSION}"/m' src-tauri/Cargo.toml
(cd src-tauri && cargo update --workspace --quiet)  # 同步 Cargo.lock 里的自身版本
git add package.json package-lock.json src-tauri/tauri.conf.json \
        src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -q -m "chore: bump version to $version"

# ---------- 本地校验 ----------
if [[ -z ${SKIP_CHECKS:-} ]]; then
    echo "==> cargo test"
    (cd src-tauri && cargo test --quiet)
    echo "==> npm run build"
    npm run build >/dev/null
fi

# ---------- 推送 main ----------
echo "==> 推送 main"
git_remote push main

# ---------- 合并进 release 并推送（触发 workflow） ----------
echo "==> 更新 release 分支"
git checkout -q release
if ! git merge --ff-only origin/release >/dev/null 2>&1; then
    echo "错误：本地 release 与远程分叉，请手动处理后重试。" >&2
    exit 1
fi

if ! git merge main -m "merge: main into release for build trigger"; then
    git merge --abort
    echo "错误：main 合并进 release 有冲突，请手动解决后自行推送 release。" >&2
    exit 1
fi

echo "==> 推送 release（触发 GitHub Actions）"
git_remote push release

if command -v gh >/dev/null; then
    sleep 5
    echo "==> 最新的 workflow 运行："
    gh run list --limit 1
    echo "跟踪进度：gh run watch \$(gh run list --limit 1 --json databaseId -q '.[0].databaseId')"
fi

echo "完成。构建产物会以 draft release 形式出现在 GitHub Releases，需手动 publish。"
