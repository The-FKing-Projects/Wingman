#!/usr/bin/env bash
set -euo pipefail

if [[ $# != 2 ]]; then
  echo "Usage: $0 ARTIFACT_DIRECTORY EXPECTED_VERSION" >&2
  exit 1
fi

verifier="$(cd "$(dirname "$0")" && pwd)/check_packages.sh"
artifacts=$(realpath "$1")
version=$2
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT

expect_failure() {
  local directory=$1 expected_version=$2 message=$3
  if bash "$verifier" "$directory" "$expected_version" > "$work/result.txt" 2>&1; then
    echo "Package verification unexpectedly accepted: $message" >&2
    exit 1
  fi
  cat "$work/result.txt"
  grep -Fq "$message" "$work/result.txt"
}

expect_failure "$artifacts" invalid-version "DEB version: expected"

mkdir "$work/incomplete"
cp -l -- "$artifacts"/*.deb "$artifacts"/*.rpm "$artifacts"/*.AppImage "$work/incomplete/"
grep -v '\.AppImage$' "$artifacts/SHA256SUMS" > "$work/incomplete/SHA256SUMS"
expect_failure "$work/incomplete" "$version" "Checksum manifest must cover exactly"

mkdir "$work/bad-digest"
cp -l -- "$artifacts"/*.deb "$artifacts"/*.rpm "$artifacts"/*.AppImage "$work/bad-digest/"
sed 's/^[0-9a-f]/x/' "$artifacts/SHA256SUMS" > "$work/bad-digest/SHA256SUMS"
expect_failure "$work/bad-digest" "$version" "Checksum manifest must cover exactly"

mkdir "$work/stale-image"
cp -l -- "$artifacts"/*.deb "$artifacts"/*.rpm "$work/stale-image/"
cp -l -- "$artifacts"/*.AppImage "$work/stale-image/Wingman_0.0.0_amd64.AppImage"
(
  cd "$work/stale-image"
  sha256sum -- *.deb *.rpm *.AppImage > SHA256SUMS
)
expect_failure "$work/stale-image" "$version" "AppImage filename: expected"

echo "Rejected wrong versions, incomplete manifests, bad digests, and stale AppImage filenames"
