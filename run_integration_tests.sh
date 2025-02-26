#!/bin/bash
set -e

# Install cargo-nextest if not already installed
if ! command -v cargo-nextest &> /dev/null; then
    echo "Installing cargo-nextest..."
    cargo install cargo-nextest
fi

# Build the tests
echo "Building tests..."
cargo build --tests

# Run the integration tests using nextest
echo "Running integration tests..."
cargo nextest run --profile integration

# Report success
echo "All integration tests passed!"