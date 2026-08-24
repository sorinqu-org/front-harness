#!/usr/bin/env bash
set -euo pipefail

echo "=========================================================="
echo " FrontHarness POSIX Installer"
echo " Automated Frontend Generation & Redesign CLI/TUI"
echo "=========================================================="

INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/frontharness"
mkdir -p "${INSTALL_DIR}" "${CONFIG_DIR}"

# 1. Dependency checks
command -v node >/dev/null 2>&1 || { echo "[WARN] Node.js is not installed. Please install Node.js (v18+) for Playwright crawling."; }
command -v git >/dev/null 2>&1 || { echo "[WARN] Git is not installed. Please install Git."; }
command -v cargo >/dev/null 2>&1 || { echo "[ERROR] Cargo/Rust is required to build FrontHarness. Install via https://rustup.rs"; exit 1; }

# 2. Build FrontHarness
echo "[Installer] Compiling FrontHarness release binary..."
cargo build --release

# 3. Install binary
cp -f target/release/frontharness "${INSTALL_DIR}/frontharness"
chmod +x "${INSTALL_DIR}/frontharness"
ln -sf "${INSTALL_DIR}/frontharness" "${INSTALL_DIR}/fh"

echo "[Installer] Installed binary to ${INSTALL_DIR}/frontharness (alias: ${INSTALL_DIR}/fh)"

# 4. PATH Configuration for active shell
SHELL_NAME=$(basename "${SHELL:-bash}")
SHELL_CONFIG=""

case "${SHELL_NAME}" in
    bash)
        SHELL_CONFIG="${HOME}/.bashrc"
        ;;
    zsh)
        SHELL_CONFIG="${HOME}/.zshrc"
        ;;
    fish)
        SHELL_CONFIG="${HOME}/.config/fish/config.fish"
        ;;
    *)
        SHELL_CONFIG="${HOME}/.profile"
        ;;
esac

if [ -f "${SHELL_CONFIG}" ]; then
    if ! grep -q ".local/bin" "${SHELL_CONFIG}"; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "${SHELL_CONFIG}"
        echo "[Installer] Added \$HOME/.local/bin to ${SHELL_CONFIG}"
    fi
fi

# 5. Default configuration template
if [ ! -f "${CONFIG_DIR}/config.yaml" ]; then
    cat << 'CONF' > "${CONFIG_DIR}/config.yaml"
llm:
  base_url: "https://agentrouter.org/v1"
  model: "gpt-5.6-sol"
  reasoning_effort: "high"
  timeout_seconds: 120
browser:
  headless: true
  dev_server_port: 3000
CONF
    echo "[Installer] Created default config at ${CONFIG_DIR}/config.yaml"
fi

echo "=========================================================="
echo " FrontHarness installation completed successfully!"
echo " Run 'frontharness --help' or 'fh' to start."
echo "=========================================================="
