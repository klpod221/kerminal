#!/bin/bash
# ------------------------------------------
# Script to recreate and push git tag
# Written by klpod221 - github.com/klpod221
# ------------------------------------------

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <tag-name>"
    echo "Example: $0 v2.1.3"
    exit 1
fi

TAG="$1"

# Validate tag format
if [[ ! "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "⚠️  Warning: Tag should follow format vX.Y.Z (e.g., v2.1.3)"
    read -p "Continue anyway? (y/n): " CONTINUE
    if [ "$CONTINUE" != "y" ]; then
        exit 0
    fi
fi

echo "=========================================="
echo "  Recreating Git Tag: $TAG"
echo "=========================================="
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
git tag "$TAG"

echo "📤 Pushing tag to remote..."
git push origin "$TAG"

echo ""
echo "✅ Done! Tag $TAG has been recreated and pushed."
echo "🚀 GitHub Actions workflow should start running soon."
echo ""
echo "View the workflow at:"
echo "https://github.com/klpod221/kerminal/actions"
