#!/bin/bash
# ------------------------------------------
# Script to generate Android keystore for Kerminal
# Written by klpod221 - github.com/klpod221
# ------------------------------------------

echo "=========================================="
echo "  Kerminal Android Keystore Generator"
echo "=========================================="
echo ""

KEYSTORE_FILE=".android/kerminal-release.keystore"

if [[ -f "$KEYSTORE_FILE" ]]; then
    echo "⚠️  Keystore already exists at: $KEYSTORE_FILE"
    read -p "Do you want to overwrite it? (yes/no): " OVERWRITE
    if [[ "$OVERWRITE" != "yes" ]]; then
        echo "Aborted."
        exit 0
    fi
    rm "$KEYSTORE_FILE"
fi

echo "Please enter keystore information:"
echo ""

read -sp "Store Password: " STORE_PASS
echo ""
read -sp "Key Password: " KEY_PASS
echo ""
echo ""

echo "Generating keystore..."
keytool -genkey -v -keystore "$KEYSTORE_FILE" \
    -alias kerminal \
    -keyalg RSA \
    -keysize 2048 \
    -validity 10000 \
    -storepass "$STORE_PASS" \
    -keypass "$KEY_PASS"

if [[ $? -eq 0 ]]; then
    echo ""
    echo "✅ Keystore created successfully at: $KEYSTORE_FILE"
    echo ""
    echo "📋 Next steps:"
    echo "1. Convert keystore to base64:"
    echo "   base64 -w 0 $KEYSTORE_FILE"
    echo ""
    echo "2. Add these secrets to GitHub:"
    echo "   - ANDROID_KEYSTORE_BASE64: (output from step 1)"
    echo "   - ANDROID_KEYSTORE_PASSWORD: $STORE_PASS"
    echo "   - ANDROID_KEY_PASSWORD: $KEY_PASS"
    echo "   - ANDROID_KEY_ALIAS: kerminal"
    echo ""
    echo "⚠️  IMPORTANT: Backup this keystore safely!"
    echo "   If you lose it, you cannot update your app!"
else
    echo ""
    echo "❌ Failed to create keystore"
    exit 1
fi
