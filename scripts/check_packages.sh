#!/usr/bin/env bash
set -euo pipefail

if [[ $# != 2 ]]; then
  echo "Usage: $0 ARTIFACT_DIRECTORY EXPECTED_VERSION" >&2
  exit 1
fi

for tool in dpkg-deb rpm bsdtar readelf desktop-file-validate sha256sum; do
  command -v "$tool" >/dev/null || {
    echo "Missing required package inspection tool: $tool" >&2
    exit 1
  }
done

artifacts=$(realpath "$1")
version=$2
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT

fail() {
  echo "Package verification failed: $*" >&2
  exit 1
}

require_equal() {
  [[ "$1" == "$2" ]] || fail "$3: expected '$2', found '$1'"
}

check_tree() {
  local root=$1 desktop=$2 binary=$3 icon=$4
  [[ -f "$desktop" ]] || fail "Missing desktop entry: $desktop"
  desktop-file-validate "$desktop"
  grep -qx 'Name=Wingman' "$desktop" || fail "Incorrect launcher name"
  grep -qx 'Exec=wingman' "$desktop" || fail "Unexpected launcher command"
  grep -qx 'Icon=wingman' "$desktop" || fail "Incorrect launcher icon"
  [[ -s "$icon" ]] || fail "Missing launcher icon in $root"
  [[ -f "$binary" && -x "$binary" ]] || fail "Missing executable in $root"
  readelf -h "$binary" | grep -q 'Class:.*ELF64' || fail "Non-64-bit executable"
  readelf -h "$binary" | grep -q 'Machine:.*Advanced Micro Devices X86-64' \
    || fail "Non-x86_64 executable"
}

cd "$artifacts"
shopt -s nullglob
debs=(*.deb)
rpms=(*.rpm)
images=(*.AppImage)
[[ ${#debs[@]} == 1 && ${#rpms[@]} == 1 && ${#images[@]} == 1 ]] \
  || fail "Expected exactly one DEB, RPM, and AppImage"
[[ -f SHA256SUMS ]] || fail "Missing checksum manifest"
expected_manifest=$(sha256sum -- "${debs[@]}" "${rpms[@]}" "${images[@]}" | LC_ALL=C sort)
diff -u <(printf '%s\n' "$expected_manifest") <(LC_ALL=C sort SHA256SUMS) \
  || fail "Checksum manifest must cover exactly the three packages with matching hashes"
sha256sum -c SHA256SUMS

deb=${debs[0]}
require_equal "$(dpkg-deb -f "$deb" Package)" wingman "DEB name"
require_equal "$(dpkg-deb -f "$deb" Version)" "$version" "DEB version"
require_equal "$(dpkg-deb -f "$deb" Architecture)" amd64 "DEB architecture"
dependencies=$(dpkg-deb -f "$deb" Depends)
for library in libwebkit2gtk-4.1-0 libgtk-3-0; do
  [[ "$dependencies" == *"$library"* ]] || fail "DEB missing dependency $library"
done
dpkg-deb -x "$deb" "$work/deb"
check_tree "$work/deb" "$work/deb/usr/share/applications/Wingman.desktop" \
  "$work/deb/usr/bin/wingman" \
  "$work/deb/usr/share/icons/hicolor/128x128/apps/wingman.png"
echo "DEB metadata, dependencies, launcher, icon, and ELF architecture verified"

rpm=${rpms[0]}
require_equal "$(rpm -qp --qf '%{NAME}' "$rpm")" wingman "RPM name"
require_equal "$(rpm -qp --qf '%{VERSION}' "$rpm")" "$version" "RPM version"
require_equal "$(rpm -qp --qf '%{ARCH}' "$rpm")" x86_64 "RPM architecture"
if ! dependencies=$(rpm -qp --requires "$rpm"); then
  printf '%s\n' "$dependencies" >&2
  fail "Could not read RPM dependencies"
fi
for library in libwebkit2gtk-4.1.so.0 libgtk-3.so.0; do
  [[ "$dependencies" == *"$library"* ]] || fail "RPM missing dependency $library"
done
mkdir "$work/rpm"
bsdtar -xf "$rpm" -C "$work/rpm"
check_tree "$work/rpm" "$work/rpm/usr/share/applications/Wingman.desktop" \
  "$work/rpm/usr/bin/wingman" \
  "$work/rpm/usr/share/icons/hicolor/128x128/apps/wingman.png"
echo "RPM metadata, dependencies, launcher, icon, and ELF architecture verified"

image="$artifacts/${images[0]}"
require_equal "${images[0]}" "Wingman_${version}_amd64.AppImage" "AppImage filename"
# Only the AppImage runtime's extraction mode runs; never launch the application.
chmod +x "$image"
(cd "$work" && "$image" --appimage-extract > /dev/null)
check_tree "$work/squashfs-root" "$work/squashfs-root/Wingman.desktop" \
  "$work/squashfs-root/usr/bin/wingman" "$work/squashfs-root/Wingman.png"
[[ -x "$work/squashfs-root/AppRun" ]] || fail "Missing AppImage entry point"
echo "AppImage entry point, launcher, icon, and ELF architecture verified"
echo "Static package checks passed; installation and GUI behavior require separate testing"
