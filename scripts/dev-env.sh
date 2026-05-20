#!/usr/bin/env sh

# Normalise the local development environment for Tauri builds.
# CodexMonitor runs from an AppImage, which can leak library paths into child
# processes. Keep project builds on system GLib/WebKit and rustup Rust.

_is_sourced=1
case "$0" in
  *dev-env.sh) _is_sourced=0 ;;
esac

unset LD_LIBRARY_PATH
unset APPDIR
unset APPIMAGE

_prepend_path() {
  if [ -n "$1" ] && [ -d "$1" ]; then
    PATH="$1:$PATH"
  fi
}

_node_bin="${UNIFIKATOR_NODE_BIN:-}"
if [ -z "$_node_bin" ]; then
  if [ -x "$HOME/.nvm/versions/node/v25.9.0/bin/node" ]; then
    _node_bin="$HOME/.nvm/versions/node/v25.9.0/bin"
  else
    for _candidate in "$HOME"/.nvm/versions/node/*/bin; do
      if [ -x "$_candidate/node" ]; then
        _node_bin="$_candidate"
      fi
    done
  fi
fi

# Put system tools before Homebrew so glib-compile-schemas comes from /usr/bin.
PATH="/usr/local/bin:/usr/bin:/bin:$PATH"
_prepend_path "$_node_bin"
_prepend_path "$HOME/.cargo/bin"
export PATH

if [ "$_is_sourced" -eq 0 ]; then
  if [ "$#" -gt 0 ]; then
    exec "$@"
  fi

  printf '%s\n' "Unifikator development environment"
  printf 'node: '; command -v node || true
  node --version 2>/dev/null || true
  printf 'pnpm: '; command -v pnpm || true
  pnpm --version 2>/dev/null || true
  printf 'rustc: '; command -v rustc || true
  rustc --version 2>/dev/null || true
  printf 'cargo: '; command -v cargo || true
  cargo --version 2>/dev/null || true
  printf 'glib-compile-schemas: '; command -v glib-compile-schemas || true
  glib-compile-schemas --version 2>/dev/null || true
fi
