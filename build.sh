#!/bin/bash

echo "🦀 Building FastyFileManager for Linux/macOS..."

cargo build --release

if [ $? -ne 0 ]; then
  echo "❌ Build failed!"
  exit 1
fi

BINARY="target/release/ffm"
INSTALL_DIR="$HOME/.local/bin"

echo "✅ Build complete!"
echo "📦 Size: $(du -h $BINARY | cut -f1)"

# Копируем бинарник
mkdir -p "$INSTALL_DIR"
cp "$BINARY" "$INSTALL_DIR/ffm"
echo "📌 Installed to: $INSTALL_DIR/ffm"

# Добавляем в PATH если ещё нет
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  SHELL_RC=""
  if [ -f "$HOME/.zshrc" ]; then
    SHELL_RC="$HOME/.zshrc"
  elif [ -f "$HOME/.bashrc" ]; then
    SHELL_RC="$HOME/.bashrc"
  fi

  if [ -n "$SHELL_RC" ]; then
    echo "" >> "$SHELL_RC"
    echo "# FastyFileManager" >> "$SHELL_RC"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"
    echo "📌 Added $INSTALL_DIR to PATH in $SHELL_RC"
    echo "   Restart your terminal or run: source $SHELL_RC"
  else
    echo "⚠️  Could not detect shell config. Add manually:"
    echo "   export PATH=\"$INSTALL_DIR:\$PATH\""
  fi
else
  echo "📌 Already in PATH"
fi

echo "🚀 Run: ffm"
