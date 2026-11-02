#!/bin/bash
# ------------------------------------------
# Script to update version and recrete tag to trigger GitHub Actions
# Written by klpod221 - github.com/klpod221
# ------------------------------------------

set -e

SEPARATOR="=========================================="

if [[ -z "$1" ]]; then
    echo "Usage: $0 <tag-name>"
    echo "Example: $0 v2.1.3"
    exit 1
fi

TAG="$1"

# Validate tag format
if [[ ! "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "⚠️  Warning: Tag should follow format vX.Y.Z (e.g., v2.1.3)"
    read -p "Continue anyway? (y/n): " CONTINUE
    if [[ "$CONTINUE" != "y" ]]; then
        exit 0
    fi
fi

echo "$SEPARATOR"
echo "  Update Version to match Tag: $TAG"
echo "$SEPARATOR"
echo ""

VERSION="${TAG:1}"  # Remove the 'v' prefix
echo "🔄 Updating version to $VERSION in relevant files..."
# Update version in package.json
sed -i.bak -E "s/\"version\": \"[0-9]+\.[0-9]+\.[0-9]+\"/\"version\": \"$VERSION\"/" package.json

# Update version in src/core/Cargo.toml
sed -i.bak -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"$VERSION\"/" src/core/Cargo.toml

# Update version in src/tauri/Cargo.toml
sed -i.bak -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"$VERSION\"/" src/tauri/Cargo.toml

# update version in src/tauri/tauri.conf.json
sed -i.bak -E "s/\"version\": \"[0-9]+\.[0-9]+\.[0-9]+\"/\"version\": \"$VERSION\"/" src/tauri/tauri.conf.json

# Clean up backup files
rm -f package.json.bak src/core/Cargo.toml.bak src/tauri/Cargo.toml.bak src/tauri/tauri.conf.json.bak

# Update lock files
npm install --package-lock-only
cargo check --workspace

echo "$SEPARATOR"
echo "  Committing Version Update"
echo "$SEPARATOR"
echo ""

git add .

# Check if there are any changes to commit
if git diff --cached --quiet && git diff --quiet; then
    echo "ℹ️  No changes to commit. Skipping commit and push."
else
    echo "📝 Committing changes..."
    git commit -m "chore: update version to $VERSION"
    echo "📤 Pushing to origin main..."
    git push origin main > /dev/null
    echo "✅ Changes committed and pushed successfully."
fi

echo "$SEPARATOR"
echo "  Recreating Git Tag: $TAG"
echo "$SEPARATOR"
echo ""

# Check if tag exists locally
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "🗑️  Deleting local tag: $TAG"
    git tag -d "$TAG"
else
    echo "ℹ️  Local tag doesn't exist"
fi

# Check if tag exists on remote
if git ls-remote --tags origin | grep -q "refs/tags/$TAG"; then
    echo "🗑️  Deleting remote tag: $TAG"
    git push origin ":refs/tags/$TAG"
else
    echo "ℹ️  Remote tag doesn't exist"
fi

echo ""
echo "✨ Creating new tag: $TAG"
git tag -f "$TAG"

echo "📤 Pushing tag to remote..."
git push origin "$TAG" --force

echo ""
echo "✅ Done! Version updated and tag recreated."
echo ""
echo "🚀 GitHub Actions workflow should start running soon."
echo ""
echo "View the workflow at:"
echo "https://github.com/klpod221/kerminal/actions"
