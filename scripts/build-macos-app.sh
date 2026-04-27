#!/usr/bin/env bash
# Build a macOS .app bundle for moai-studio.
#
# Usage:
#   scripts/build-macos-app.sh                # build + bundle
#   scripts/build-macos-app.sh --install      # also copy to ~/Applications
#   scripts/build-macos-app.sh --skip-build   # skip cargo build, only bundle
#   scripts/build-macos-app.sh --debug        # use debug profile instead of release
#
# Output: target/<profile>/moai-studio.app
#
# This script does NOT depend on cargo-bundle. It hand-crafts the bundle so it
# works on any system with cargo + standard Unix tools.
#
# Bundle layout:
#   moai-studio.app/
#     Contents/
#       Info.plist
#       MacOS/
#         moai-studio                  (release binary)
#         libghostty-vt.dylib          (FFI dep, resolved via @executable_path rpath)
#       Resources/
#         moai-studio.png              (icon placeholder; replace with .icns for GA)
#       _CodeSignature/
#         (created by codesign --sign - --force --deep)
set -euo pipefail

# ---- Resolve repo root --------------------------------------------------------
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# ---- Parse flags --------------------------------------------------------------
PROFILE="release"
DO_BUILD=1
DO_INSTALL=0
for arg in "$@"; do
    case "${arg}" in
        --install) DO_INSTALL=1 ;;
        --skip-build) DO_BUILD=0 ;;
        --debug) PROFILE="debug" ;;
        --help|-h)
            sed -n '2,/^set -euo/p' "$0" | sed 's/^# *//;s/^#$//' | sed '$d'
            exit 0
            ;;
        *) echo "Unknown flag: ${arg}" >&2; exit 2 ;;
    esac
done

# ---- Sanity ------------------------------------------------------------------
if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "ERROR: this script is macOS-only (uname=$(uname -s))" >&2
    exit 1
fi

PLIST_TEMPLATE="${REPO_ROOT}/assets/macos/Info.plist.template"
ICON_PNG="${REPO_ROOT}/assets/icons/256x256/moai-studio.png"
if [[ ! -f "${PLIST_TEMPLATE}" ]]; then
    echo "ERROR: Info.plist template missing at ${PLIST_TEMPLATE}" >&2
    exit 1
fi
if [[ ! -f "${ICON_PNG}" ]]; then
    echo "ERROR: Icon missing at ${ICON_PNG}" >&2
    exit 1
fi

# ---- Build -------------------------------------------------------------------
if [[ "${DO_BUILD}" -eq 1 ]]; then
    echo "==> cargo build (${PROFILE})"
    if [[ "${PROFILE}" == "release" ]]; then
        cargo build --release -p moai-studio-app
    else
        cargo build -p moai-studio-app
    fi
fi

BIN="${REPO_ROOT}/target/${PROFILE}/moai-studio"
DYLIB="${REPO_ROOT}/target/${PROFILE}/libghostty-vt.dylib"
if [[ ! -x "${BIN}" ]]; then
    echo "ERROR: binary not found at ${BIN} (run without --skip-build)" >&2
    exit 1
fi
if [[ ! -f "${DYLIB}" ]]; then
    echo "ERROR: dylib not found at ${DYLIB}" >&2
    exit 1
fi

# ---- Resolve version from workspace Cargo.toml -------------------------------
# crates/moai-studio-app uses `version.workspace = true`, so read the workspace root.
CARGO_VERSION="$(awk '/^\[workspace.package\]/{f=1; next} /^\[/{f=0} f && /^version *=/' "${REPO_ROOT}/Cargo.toml" | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
if [[ -z "${CARGO_VERSION}" ]]; then
    echo "ERROR: could not parse version from Cargo.toml" >&2
    exit 1
fi
echo "==> Version: ${CARGO_VERSION}"

# ---- Assemble bundle ---------------------------------------------------------
APP_DIR="${REPO_ROOT}/target/${PROFILE}/moai-studio.app"
echo "==> Building bundle at ${APP_DIR}"
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

# Info.plist with version substitution
sed "s/__VERSION__/${CARGO_VERSION}/g" "${PLIST_TEMPLATE}" > "${APP_DIR}/Contents/Info.plist"

# Binary + dylib (dylib resolved via @executable_path rpath, see otool -l of moai-studio)
cp "${BIN}" "${APP_DIR}/Contents/MacOS/moai-studio"
cp "${DYLIB}" "${APP_DIR}/Contents/MacOS/libghostty-vt.dylib"

# Icon (PNG placeholder; replace with .icns for GA)
cp "${ICON_PNG}" "${APP_DIR}/Contents/Resources/moai-studio.png"

# ---- Ad-hoc code sign --------------------------------------------------------
echo "==> Ad-hoc codesign"
codesign --force --deep --sign - "${APP_DIR}" 2>&1 | tail -5

# ---- Install (optional) ------------------------------------------------------
if [[ "${DO_INSTALL}" -eq 1 ]]; then
    INSTALL_DEST="${HOME}/Applications/moai-studio.app"
    echo "==> Installing to ${INSTALL_DEST}"
    # Kill any running instance to avoid stale binary lock
    pkill -f "moai-studio.app/Contents/MacOS" 2>/dev/null || true
    sleep 1
    rm -rf "${INSTALL_DEST}"
    mkdir -p "$(dirname "${INSTALL_DEST}")"
    cp -R "${APP_DIR}" "${INSTALL_DEST}"
fi

echo ""
echo "Built: ${APP_DIR}"
if [[ "${DO_INSTALL}" -eq 1 ]]; then
    echo "Installed: ${HOME}/Applications/moai-studio.app"
    echo "Run: open ~/Applications/moai-studio.app"
else
    echo "Run: open ${APP_DIR}"
fi
