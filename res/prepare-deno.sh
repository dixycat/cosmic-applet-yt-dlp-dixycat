#!/usr/bin/env bash
# Download the pinned Deno runtime used by yt-dlp's YouTube challenge solver.
set -e

deno_version="v2.9.6"
case "$(uname -m)" in
    x86_64) deno_target="x86_64-unknown-linux-gnu" ;;
    aarch64) deno_target="aarch64-unknown-linux-gnu" ;;
    *)
        echo "Unsupported architecture for bundled Deno: $(uname -m)" >&2
        exit 1
        ;;
esac

if [[ -x res/deno ]] && ./res/deno --version 2>/dev/null | head -n1 | grep -Fq " ${deno_version#v} "; then
    echo "Using existing bundled Deno runtime:"
    ./res/deno --version
    exit 0
fi

for required in curl unzip; do
    if ! command -v "$required" >/dev/null 2>&1; then
        echo "Missing required tool: $required" >&2
        exit 1
    fi
done

temp_dir="$(mktemp -d)"
trap 'rm -rf "$temp_dir"' EXIT
archive="deno-${deno_target}.zip"
url="https://github.com/denoland/deno/releases/download/${deno_version}/${archive}"

echo "Downloading Deno ${deno_version} (${deno_target})"
curl --fail --location --retry 3 --proto '=https' --tlsv1.2 \
    --output "$temp_dir/$archive" "$url"
unzip -q "$temp_dir/$archive" -d "$temp_dir"
install -Dm0755 "$temp_dir/deno" "res/deno"
echo "Bundled Deno runtime:"
./res/deno --version
