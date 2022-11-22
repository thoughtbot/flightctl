#!/bin/sh

set -e

FLIGHTCTL_VERSION=0.2.1
PREFIX=$(dirname "$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)")
FLIGHTCTL_TMP="$PREFIX/tmp"
FLIGHTCTL_BIN="$FLIGHTCTL_TMP/flightctl-$FLIGHTCTL_VERSION"
FLIGHTCTL_OS=$(uname 2>/dev/null || echo unknown)
FLIGHTCTL_ARCH="$(uname -m 2>/dev/null || echo unknown)"
FLIGHTCTL_REPO="https://github.com/thoughtbot/flightctl"

if [ ! -f "$FLIGHTCTL_BIN" ]; then
  echo "Installing flightctl $FLIGHTCTL_VERSION..." >&2
  echo "Operation system: $FLIGHTCTL_OS" >&2
  echo "Architecture: $FLIGHTCTL_ARCH" >&2
  mkdir -p "$FLIGHTCTL_TMP" >&2

  case "$FLIGHTCTL_OS/$FLIGHTCTL_ARCH" in
    Linux/x86_64)
      FLIGHTCTL_ASSET="flightctl-v$FLIGHTCTL_VERSION-x86_64-unknown-linux-musl"
      ;;
    Linux/aarch64)
      FLIGHTCTL_ASSET="flightctl-v$FLIGHTCTL_VERSION-aarch64-unknown-linux-musl"
      ;;
    Darwin/x86_64)
      FLIGHTCTL_ASSET="flightctl-v$FLIGHTCTL_VERSION-x86_64-apple-darwin"
      ;;
    Darwin/arm64)
      FLIGHTCTL_ASSET="flightctl-v$FLIGHTCTL_VERSION-aarch64-apple-darwin"
      ;;
    *)
      echo "FATAL: Unsupported operating system or architecture." >&2
      exit 1
      ;;
  esac

  FLIGHTCTL_ARCHIVE="$FLIGHTCTL_ASSET.tar.gz"
  FLIGHTCTL_URL="$FLIGHTCTL_REPO/releases/download/v$FLIGHTCTL_VERSION/$FLIGHTCTL_ARCHIVE"
  echo "Downloading $FLIGHTCTL_URL..." >&2
  curl --location "$FLIGHTCTL_URL" --output "$FLIGHTCTL_TMP/$FLIGHTCTL_ARCHIVE"
  echo "Extracting..." >&2
  tar vzxf "$FLIGHTCTL_TMP/$FLIGHTCTL_ARCHIVE" -C "$FLIGHTCTL_TMP"
  cp "$FLIGHTCTL_TMP/$FLIGHTCTL_ASSET/flightctl" "$FLIGHTCTL_BIN"
  echo "Cleaning up..." >&2
  rm "$FLIGHTCTL_TMP/$FLIGHTCTL_ARCHIVE"
  rm -r "$FLIGHTCTL_TMP/$FLIGHTCTL_ASSET"
  echo "" >&2
fi

exec "$FLIGHTCTL_BIN" "$@"
