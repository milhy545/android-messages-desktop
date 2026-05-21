#!/usr/bin/env bash
set -eo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# shellcheck source=dev-env.sh
. "$ROOT_DIR/scripts/dev-env.sh"

run_install=1
run_no_bundle=1
run_appimage=0
use_tmp_target=0

usage() {
  cat <<'EOF'
Usage: scripts/check-local.sh [options]

Runs the local Unifikator verification workflow in a sanitised Tauri
environment.

Options:
  --quick             Install dependencies, run cargo check, build without bundle. Default.
  --bundle-appimage   Build the Linux AppImage bundle after cargo check.
  --skip-install      Do not run pnpm install --frozen-lockfile.
  --tmp-target        Use /tmp/unifikator-tauri-target as CARGO_TARGET_DIR.
  -h, --help          Show this help.
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --quick)
      run_no_bundle=1
      run_appimage=0
      ;;
    --bundle-appimage)
      run_no_bundle=0
      run_appimage=1
      ;;
    --skip-install)
      run_install=0
      ;;
    --tmp-target)
      use_tmp_target=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown option: %s\n\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

if [ "$use_tmp_target" -eq 1 ] && [ -z "$CARGO_TARGET_DIR" ]; then
  export CARGO_TARGET_DIR="/tmp/unifikator-tauri-target"
fi

step() {
  printf '\n==> %s\n' "$*"
}

run() {
  step "$*"
  "$@"
}

run_in_dir() {
  dir="$1"
  shift
  step "cd $dir && $*"
  (cd "$dir" && "$@")
}

step "Environment"
printf 'root: %s\n' "$ROOT_DIR"
printf 'node: %s (%s)\n' "$(command -v node)" "$(node --version)"
printf 'pnpm: %s (%s)\n' "$(command -v pnpm)" "$(pnpm --version)"
printf 'rustc: %s (%s)\n' "$(command -v rustc)" "$(rustc --version)"
printf 'cargo: %s (%s)\n' "$(command -v cargo)" "$(cargo --version)"
printf 'glib-compile-schemas: %s (%s)\n' "$(command -v glib-compile-schemas)" "$(glib-compile-schemas --version)"
printf 'webkit2gtk-4.1: %s\n' "$(pkg-config --modversion webkit2gtk-4.1)"
printf 'gtk+-3.0: %s\n' "$(pkg-config --modversion gtk+-3.0)"
if [ -n "$CARGO_TARGET_DIR" ]; then
  printf 'CARGO_TARGET_DIR: %s\n' "$CARGO_TARGET_DIR"
fi
df -h "$ROOT_DIR" /tmp

if [ "$run_install" -eq 1 ]; then
  run pnpm install --frozen-lockfile
fi

run_in_dir "$ROOT_DIR/src-tauri" cargo check

if [ "$run_no_bundle" -eq 1 ]; then
  run pnpm tauri build --no-bundle
fi

if [ "$run_appimage" -eq 1 ]; then
  run pnpm tauri build --bundles appimage
  target_dir="$ROOT_DIR/src-tauri/target"
  if [ -n "$CARGO_TARGET_DIR" ]; then
    target_dir="$CARGO_TARGET_DIR"
  fi
  step "Built AppImage"
  find "$target_dir/release/bundle/appimage" \
    -maxdepth 1 -type f -name '*.AppImage' -print -exec ls -lh {} \;
fi

step "Local check completed"
