#!/bin/bash
set -e

echo "Release script"
echo "=============================="

# Get current version
CURRENT=$(grep -m1 '^version =' Cargo.toml | cut -d'"' -f2)
echo "Current version: $CURRENT"

# Get git history for changelog
echo ""
echo "Getting commit history since last release..."
PREVIOUS_TAG=$(git describe --tags --abbrev=0 HEAD~1 2>/dev/null || echo "")

if [ -n "$PREVIOUS_TAG" ]; then
    echo "Previous tag: $PREVIOUS_TAG"
    COMMITS=$(git log --pretty=format:"%s" --no-merges "${PREVIOUS_TAG}..HEAD")
else
    echo "No previous tag found (first release?)"
    COMMITS=$(git log --pretty=format:"%s" --no-merges)
fi

echo ""
echo "=== Commits since last release ==="
if [ -z "$COMMITS" ]; then
    echo "No commits found"
else
    # Show commits with numbers
    I=1
    while IFS= read -r line; do
        echo "$I) $line"
        I=$((I + 1))
    done <<< "$COMMITS"
fi

echo ""
echo "=== Formatting commits ==="
echo "Enter commit messages to include in changelog (separated by semicolon)"
echo "Example: fix: crash on startup;feat: add new level;docs: update readme"
echo "Or press Enter to use all commits as-is"
read -p "Commit messages: " COMMIT_INPUT

if [ -z "$COMMIT_INPUT" ]; then
    # Use all commits
    FORMATTED_COMMITS="$COMMITS"
else
    # Use user input
    FORMATTED_COMMITS=$(echo "$COMMIT_INPUT" | tr ';' '\n')
fi

# Convert to bullet points
BULLET_COMMITS=""
if [ -n "$FORMATTED_COMMITS" ]; then
    while IFS= read -r line; do
        if [ -n "$line" ]; then
            BULLET_COMMITS="${BULLET_COMMITS}- ${line}"$'\n'
        fi
    done <<< "$FORMATTED_COMMITS"
fi

echo ""
echo "=== Generated commit list ==="
echo "$BULLET_COMMITS"

echo ""
echo "=== What's new ==="
echo "Enter summary of new features for end users (separated by semicolon)"
echo "Example: Added new puzzle type;Improved graphics;Fixed save system"
echo "This will replace 'ADD SUMMARY' in release notes"
read -p "What's new: " SUMMARY_INPUT

# Convert summary to bullet points
BULLET_SUMMARY=""
if [ -n "$SUMMARY_INPUT" ]; then
    IFS=';' read -ra SUMMARY_ITEMS <<< "$SUMMARY_INPUT"
    for item in "${SUMMARY_ITEMS[@]}"; do
        trimmed_item=$(echo "$item" | xargs)
        if [ -n "$trimmed_item" ]; then
            BULLET_SUMMARY="${BULLET_SUMMARY}- ${trimmed_item}"$'\n'
        fi
    done
fi

echo ""
echo "=== Generated summary ==="
echo "$BULLET_SUMMARY"

echo ""
echo "=============================="
echo "Now proceeding with version selection..."

# Show options
echo ""
echo "Choose one:"
echo "1) Tag current version ($CURRENT)"
echo "2) Bump patch version ($CURRENT → $(echo $CURRENT | cut -d. -f1).$(echo $CURRENT | cut -d. -f2).$(($(echo $CURRENT | cut -d. -f3) + 1)))"
echo "3) Bump minor version ($CURRENT → $(echo $CURRENT | cut -d. -f1).$(($(echo $CURRENT | cut -d. -f2) + 1)).0)"
echo "4) Bump major version ($CURRENT → $(($(echo $CURRENT | cut -d. -f1) + 1)).0.0)"
echo "5) Enter custom version"
echo ""

read -p "Choose [1-5]: " choice

case $choice in
    1)
        NEW_VERSION="$CURRENT"
        ACTION="Tagging"
        ;;
    2)
        IFS='.' read -r major minor patch <<< "$CURRENT"
        NEW_VERSION="$major.$minor.$((patch + 1))"
        ACTION="Bumping patch"
        sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
        ;;
    3)
        IFS='.' read -r major minor patch <<< "$CURRENT"
        NEW_VERSION="$major.$((minor + 1)).0"
        ACTION="Bumping minor"
        sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
        ;;
    4)
        IFS='.' read -r major minor patch <<< "$CURRENT"
        NEW_VERSION="$((major + 1)).0.0"
        ACTION="Bumping major"
        sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
        ;;
    5)
        read -p "Enter new version (e.g., 1.2.3): " NEW_VERSION
        ACTION="Setting custom"
        sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "$ACTION v$NEW_VERSION..."

# Create the release notes file
cat > RELEASE_NOTES.md << EOF
# Matrix Visualiser v${NEW_VERSION}

## What's new
This is stuff important for the end user, basically the technical changes, but summarised and less important changes ignored.
These are the new features:

${BULLET_SUMMARY}

## Technical changes
These are all of the important new commits from this release.
This may include technical changes, architectural changes, bug fixes, etc:

${BULLET_COMMITS}

## Downloads

### Windows
- **matrix-visualiser-v${NEW_VERSION}-windows-x86_64.zip**
  1. Extract the ZIP
  2. Open the 'matrix-visualiser' folder
  3. Double-click matrix-visualiser.exe

### Linux
- **matrix-visualiser-v${NEW_VERSION}-linux-x86_64.zip**
  1. Extract the ZIP
  2. Open the 'matrix-visualiser' folder in terminal
  3. Run: \`./matrix-visualiser\`

### macOS
- **matrix-visualiser-v${NEW_VERSION}-macos.zip**
  1. Extract the ZIP
  2. Open the 'matrix-visualiser' folder
  3. Right-click 'matrix-visualiser' and Open (first time only)
  4. Or run in terminal: \`./matrix-visualiser\`
EOF

echo ""
echo "=== Release Notes Created ==="
echo "You can review them in RELEASE_NOTES.md"
echo "If you need to make changes, edit the file before continuing."

# Ask if user wants to continue
read -p "Continue with release? [y/N]: " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Release cancelled. Changes have been made to Cargo.toml"
    echo "You may need to revert them manually."
    exit 1
fi

# Commit version bump and release notes
git add Cargo.toml RELEASE_NOTES.md
git commit -m "chore: bump version to $NEW_VERSION"
git push origin master

# Create and push tag
git tag "v$NEW_VERSION"
git push origin "v$NEW_VERSION"

echo ""
echo "Successfully released v$NEW_VERSION!"
echo "GitHub Actions is now building the release..."

# Clean up (optional - you might want to keep the file for reference)
# rm -f RELEASE_NOTES.md