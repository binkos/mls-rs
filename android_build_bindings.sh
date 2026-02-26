#!/bin/bash

# Script to build mls-rs-uniffi and generate Kotlin bindings
set -e  # Exit on any error

echo "Starting build process..."

# Step 1: Change to the mls-rs-uniffi directory
cd mls-rs-uniffi

# Step 2: Build .so files for both ARM architectures
echo "Building .so files for ARM architectures..."
cargo ndk -t armeabi-v7a -t arm64-v8a -o uniffi-bindgen/jniLibs build --release

# Step 3: Change to the uniffi-bindgen directory
cd uniffi-bindgen

# Step 4: Generate Kotlin bindings using the arm64 library
echo "Generating Kotlin bindings..."
cargo run --bin uniffi-bindgen generate --library jniLibs/arm64-v8a/libmls_rs_uniffi.so --language kotlin --out-dir result/interface

echo "Build and binding generation completed successfully!"