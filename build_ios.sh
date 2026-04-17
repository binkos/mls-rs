#!/bin/bash
set -e
export PATH="/opt/homebrew/bin:$PATH"
source "$HOME/.cargo/env"

# --- Project path ---
if [ -n "$1" ]; then
  PROJ="$1"
else
  read -rp "Path to MLS sources dir (e.g. /path/to/project/GemFoundation/Sources/MLS): " PROJ
fi

if [ ! -d "$PROJ" ]; then
  echo "Error: directory '$PROJ' not found"
  exit 1
fi

TARGETS="aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin x86_64-apple-darwin"

for TARGET in $TARGETS; do
  echo "=== Building $TARGET ==="
  cargo build --release --target $TARGET -p mls-rs-uniffi
done

echo "=== Generating Swift bindings ==="
mkdir -p generated
cargo run -p uniffi-bindgen --bin uniffi-bindgen generate \
  --library target/aarch64-apple-ios/release/libmls_rs_uniffi.a \
  --language swift \
  --out-dir ./generated

echo "=== Creating fat libs ==="
lipo -create \
  target/aarch64-apple-ios-sim/release/libmls_rs_uniffi.a \
  target/x86_64-apple-ios/release/libmls_rs_uniffi.a \
  -output libmls_rs_uniffi_sim.a

lipo -create \
  target/aarch64-apple-darwin/release/libmls_rs_uniffi.a \
  target/x86_64-apple-darwin/release/libmls_rs_uniffi.a \
  -output libmls_rs_uniffi_macos.a

echo "=== Creating xcframework ==="
rm -rf mls_rs_uniffi.xcframework

for dir in headers_ios headers_sim headers_macos; do
  mkdir -p $dir
  cp generated/mls_rs_uniffiFFI.h $dir/
  cat > $dir/module.modulemap << 'MODULEMAP'
module mls_rs_uniffiFFI {
    header "mls_rs_uniffiFFI.h"
    export *
}
MODULEMAP
done

xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libmls_rs_uniffi.a \
  -headers headers_ios \
  -library libmls_rs_uniffi_sim.a \
  -headers headers_sim \
  -library libmls_rs_uniffi_macos.a \
  -headers headers_macos \
  -output mls_rs_uniffi.xcframework

echo "=== Copying to project: $PROJ ==="
rm -rf "$PROJ/Frameworks/mls_rs_uniffi.xcframework"
cp -r mls_rs_uniffi.xcframework "$PROJ/Frameworks/"
cp generated/mls_rs_uniffi.swift "$PROJ/Swift/mls_rs_uniffi.swift"

echo "=== Done! ==="
