#!/bin/bash

echo "=== Starting hello-world container ==="
echo "Using environment variable: $MESSAGE"

# Setup PATH for node-18 package
PACKAGES_DIR="${PACKAGES_DIR:-../packages}"
export PATH="$PACKAGES_DIR/node-18/bin:$PATH"

echo "🔧 Configured PATH for node-18: $PATH"
echo "📝 Node version check:"
which node

echo "🚀 Running application..."
cd content
MESSAGE="Hello, World!" node app.js

echo "=== Container finished ==="