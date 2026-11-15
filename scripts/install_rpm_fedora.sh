#!/usr/bin/env bash
# Package-level RPM install checks for a Fedora host or container.
# Proves: dnf install, rpm -V, launcher/icon files, ldd resolution, dnf remove.
# Does not prove: GUI startup, login, Wayland/X11 rendering, or desktop support.
set -euo pipefail

if [[ $# != 2 ]]; then
  echo "Usage: $0 ARTIFACT_DIRECTORY EXPECTED_VERSION" >&2
  exit 1
fi

for tool in dnf rpm ldd; do
  command -v "$tool" >/dev/null || {
    echo "Missing required tool: $tool (run on Fedora)" >&2
    exit 1
  }
done

artifacts=$(realpath "$1")
version=$2

fail() {
  echo "Fedora RPM install check failed: $*" >&2
  exit 1
}

require_equal() {
  [[ "$1" == "$2" ]] || fail "$3: expected '$2', found '$1'"
}

shopt -s nullglob
rpms=("$artifacts"/*.rpm)
[[ ${#rpms[@]} == 1 ]] || fail "Expected exactly one RPM in $artifacts, found ${#rpms[@]}"
rpm_file=${rpms[0]}

# Build artifacts are unsigned; local install must not imply signature policy.
dnf -y install --setopt=localpkg_gpgcheck=0 desktop-file-utils "$rpm_file"

require_equal "$(rpm -q --qf '%{NAME}' wingman)" wingman "Installed RPM name"
require_equal "$(rpm -q --qf '%{VERSION}' wingman)" "$version" "Installed RPM version"
require_equal "$(rpm -q --qf '%{ARCH}' wingman)" x86_64 "Installed RPM architecture"

if ! rpm -V wingman; then
  fail "rpm -V reported modified or missing files"
fi

desktop=/usr/share/applications/Wingman.desktop
[[ -f "$desktop" ]] || fail "Missing desktop entry: $desktop"
desktop-file-validate "$desktop"
grep -qx 'Name=Wingman' "$desktop" || fail "Incorrect launcher name"
grep -qx 'Exec=wingman' "$desktop" || fail "Unexpected launcher command"
grep -qx 'Icon=wingman' "$desktop" || fail "Incorrect launcher icon"
grep -qx 'StartupWMClass=Wingman' "$desktop" || fail "Launcher does not match the window class"

icon=/usr/share/icons/hicolor/128x128/apps/wingman.png
[[ -s "$icon" ]] || fail "Missing launcher icon: $icon"

binary=/usr/bin/wingman
[[ -f "$binary" && -x "$binary" ]] || fail "Missing executable: $binary"
ldd "$binary" > /tmp/wingman-native-libraries.txt
cat /tmp/wingman-native-libraries.txt
if grep -q 'not found' /tmp/wingman-native-libraries.txt; then
  fail "Installed Wingman has unresolved native libraries"
fi

echo "Native dependency versions:"
rpm -q --qf '%{NAME} %{VERSION}-%{RELEASE}\n' webkit2gtk4.1 gtk3 glib2 \
  || fail "Could not query WebKitGTK/GTK/GLib package versions"

dnf -y remove wingman
rpm -q wingman >/dev/null 2>&1 && fail "wingman still installed after dnf remove"
[[ ! -e "$binary" ]] || fail "Binary remained after package removal: $binary"
[[ ! -e "$desktop" ]] || fail "Desktop entry remained after package removal: $desktop"

echo "Fedora RPM install, integrity, launcher/icon, native libraries, and removal checks passed"
echo "No graphical session was launched; desktop runtime behavior is not established"
