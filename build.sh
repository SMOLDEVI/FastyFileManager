#!/bin/bash

echo "🦀 Building FastyFileManager for Linux..."

cargo build --release

if [ $? -eq 0 ]; then
  cp target/release/ffm ./ffm
  echo "✅ Build complete!"
  echo "📦 Size: $(du -h ./ffm | cut -f1)"
  echo "🚀 Run: ./ffm"
else
  echo "❌ Build failed!"
  exit 1
fi
