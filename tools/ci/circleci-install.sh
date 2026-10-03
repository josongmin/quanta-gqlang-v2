#!/usr/bin/env bash
set -euo pipefail

# This installer runs only on the pinned x86_64 Linux CircleCI machine image.
test "$(uname -s)" = Linux
test "$(uname -m)" = x86_64

sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev curl ca-certificates

if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs |
    sh -s -- -y --profile minimal --default-toolchain none
fi
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$HOME/.local/share/qglang-ci-tools/bin:$PATH"
rustup toolchain install 1.81.0 --profile minimal --component clippy --component rustfmt
rustup toolchain install 1.95.0 --profile minimal --component clippy
rustup toolchain install nightly-2026-09-23 --profile minimal

tool_bin="$HOME/.local/share/qglang-ci-tools/bin"
mkdir -p "$tool_bin" "$HOME/.local/bin"
install_archive_binary() {
  local url="$1" name="$2" temporary
  if [[ -x "$tool_bin/$name" ]]; then
    return
  fi
  temporary="$(mktemp -d)"
  curl --proto '=https' --tlsv1.2 -fL --retry 3 "$url" -o "$temporary/archive.tar.gz"
  tar -xzf "$temporary/archive.tar.gz" -C "$temporary"
  local found
  found="$(find "$temporary" -type f -name "$name" -print -quit)"
  test -n "$found"
  install -m 0755 "$found" "$tool_bin/$name"
  rm -rf "$temporary"
}

install_archive_binary 'https://github.com/casey/just/releases/download/1.58.0/just-1.58.0-x86_64-unknown-linux-musl.tar.gz' just
install_archive_binary 'https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-0.9.104/cargo-nextest-0.9.104-x86_64-unknown-linux-gnu.tar.gz' cargo-nextest
install_archive_binary 'https://github.com/EmbarkStudios/cargo-deny/releases/download/0.19.7/cargo-deny-0.19.7-x86_64-unknown-linux-musl.tar.gz' cargo-deny
install_archive_binary 'https://github.com/crate-ci/typos/releases/download/v1.46.3/typos-v1.46.3-x86_64-unknown-linux-musl.tar.gz' typos
if [[ ! -x "$tool_bin/taplo" ]]; then
  temporary="$(mktemp)"
  curl --proto '=https' --tlsv1.2 -fL --retry 3 \
    'https://github.com/tamasfe/taplo/releases/download/0.10.0/taplo-linux-x86_64.gz' -o "$temporary"
  gzip -dc "$temporary" > "$tool_bin/taplo"
  chmod 0755 "$tool_bin/taplo"
  rm -f "$temporary"
fi

if grep -q '^tokei ' tools/ci/toolchain.lock.toml && [[ ! -x "$tool_bin/tokei" ]]; then
  RUSTUP_TOOLCHAIN=1.95.0 cargo install tokei --version 14.0.0 --locked --root "$HOME/.local/share/qglang-ci-tools"
fi
if [[ ! -x "$tool_bin/cargo-public-api" ]]; then
  RUSTUP_TOOLCHAIN=1.95.0 cargo install cargo-public-api --version 0.51.0 --locked --root "$HOME/.local/share/qglang-ci-tools"
fi

curl --proto '=https' --tlsv1.2 -fsSL https://astral.sh/uv/0.9.11/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
[[ "$(uv --version)" == 'uv 0.9.11'* ]]
uv python install 3.9.6 3.12.12
python_path="$(uv python find 3.9.6)"
ln -sfn "$python_path" "$HOME/.local/bin/python3"
site_packages="$HOME/.local/lib/python3.9/site-packages"
mkdir -p "$site_packages"
uv pip install --python "$python_path" --target "$site_packages" \
  -r tools/lint/requirements.txt
semgrep_python="$(uv python find 3.12.12)"
semgrep_venv="$HOME/.local/share/qglang-ci-tools/semgrep-venv"
uv venv --python "$semgrep_python" "$semgrep_venv"
uv pip install --python "$semgrep_venv/bin/python" 'semgrep==1.157.0'
cat > "$HOME/.local/bin/semgrep" <<'SH'
#!/usr/bin/env bash
exec "$HOME/.local/share/qglang-ci-tools/semgrep-venv/bin/semgrep" "$@"
SH
chmod 0755 "$HOME/.local/bin/semgrep"

printf 'export PATH="%s/.local/bin:%s/.local/share/qglang-ci-tools/bin:%s/.cargo/bin:$PATH"\n' \
  "$HOME" "$HOME" "$HOME" >> "$BASH_ENV"
python3 -I -S tools/ci/python_gate.py tools/ci/doctor.py
