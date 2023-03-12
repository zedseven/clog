#!/usr/bin/env bash

# Based on the release script for git-cliff, with additional adjustments

NEWLINE=$'\n'
TAG="$1"

if [ -z "$TAG" ]; then
	>&2 echo 'Please provide the tag as an argument, in the format `vX.Y.Z`.'
	exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
	>&2 echo 'You have uncommitted changes. Please clean up first.'
	exit 1
fi


# Update the version
CARGO_VERSION_COMMENT="# Managed by release.sh"
sed "s/^version = .* $CARGO_VERSION_COMMENT$/version = \"${TAG#v}\" $CARGO_VERSION_COMMENT/" -i Cargo.toml || exit

# Run checks to ensure everything is good
cargo fmt --all --check || exit
cargo clippy || exit # This also updates Cargo.lock

# Generate the changelog
git cliff --tag "$TAG" > CHANGELOG.md || exit

# Commit the version update and new changelog
git add --all && git commit -m "misc(release): Prepare for $TAG."
git show

# Create a signed tag for the new version
git tag -s -a "$TAG" -m "Release $TAG." || exit

# Verify and show the new tag
git tag -v "$TAG" || exit

# Done
echo "New version created successfully."
echo "Create the changelog for the release notes with: "\`"git cliff --tag \"$TAG\" --unreleased --strip all"\`""
