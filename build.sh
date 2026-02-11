#!/usr/bin/env bash
set -e

# --- CONFIG ---
PROJECT="doxcer"
CARGO_TOML="Cargo.toml"
TARGET_EXE="target/release/${PROJECT}.exe"
DIST_DIR="dist"

# --- BUILD STEPS ---
clear

echo "🧹 Cleaning old builds..."
cargo clean

echo "🔍 Checking code..."
cargo check

echo "🐛 Building debug..."
cargo build

echo "🧪 Building unit tests..."
cargo build --tests

echo "🧪 Running unit tests..."
cargo test

echo "⚙️  Building release..."
cargo build --release

# --- PACKAGE BUILD OUTPUT ---
if [ ! -f "$TARGET_EXE" ]; then
  echo "❌ Error: could not find ${TARGET_EXE}"
  exit 1
fi

TIMESTAMP=$(date +'%Y-%m-%d_%H-%M-%S')
OUTPUT_DIR="${DIST_DIR}/${PROJECT}-windows-x64-${TIMESTAMP}"

mkdir -p "$OUTPUT_DIR"

echo "📦 Copying ${PROJECT}.exe to ${OUTPUT_DIR}..."
cp "$TARGET_EXE" "$OUTPUT_DIR/"

echo "✅ Build complete!"
echo "📁 Output folder: ${OUTPUT_DIR}"
