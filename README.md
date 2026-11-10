# Wingman

Wingman is a lightweight Linux desktop wrapper for Microsoft Copilot, built with
Tauri v2 and the system WebKitGTK browser engine. A single native window loads
`https://copilot.com/chat`; Microsoft provides the web interface and authentication.
An internet connection is required.

## Current release

[v0.2.1-alpha](https://github.com/The-FKing-Projects/Wingman/releases/tag/v0.2.1-alpha)
is an early testing release, not a stable release.

- Native single-window application and desktop launcher.
- Microsoft sign-in and existing conversation loading have been reported working.
- DEB, RPM, and AppImage packages are published for x86_64.
- Browser data is stored on disk, but reliable sign-in across restarts is **not**
  established.

This patch release adds automated builds and package verification, not changes to
the application's browsing or authentication behavior. The previous 0.2.0 DEB
was tested by the maintainer on Ubuntu 26.04.1; 0.2.1 has automated DEB installation
and native-library checks on Ubuntu 24.04. RPM and AppImage payloads are inspected,
but their installation and desktop runtime behavior have not yet been verified.
There is not yet a tested cross-distribution support matrix. Packages depend on
their host's native libraries; an AppImage does not guarantee compatibility with
every Linux distribution or graphics driver.

## Install

Download the package for your system from the release link above. For the existing
alpha assets:

```sh
# Debian/Ubuntu: run from the download directory.
sudo apt install ./Wingman_0.2.1_amd64.deb

# Fedora/RPM systems: available, but not yet runtime-verified.
sudo dnf install ./Wingman-0.2.1-1.x86_64.rpm

# AppImage: available, but not yet runtime-verified.
chmod +x Wingman_0.2.1_amd64.AppImage
./Wingman_0.2.1_amd64.AppImage
```

Use only the command appropriate to your distribution. Launch an installed package
from the application menu or with `wingman`. The AppImage may require FUSE 2
compatibility libraries from your distribution; see
[AppImage's FUSE guidance](https://docs.appimage.org/user-guide/troubleshooting/fuse.html).
Do not run Wingman as root or disable the WebKit sandbox to work around a launch
failure.

## Known limitations

- **Restart sign-in:** Copilot may request sign-in after the app restarts.
  Persistent cookies and storage do not prove that Microsoft accepts a saved
  session. The cause remains under investigation in
  [#8](https://github.com/The-FKing-Projects/Wingman/issues/8).
- **Shutdown crashes:** a WebKit child process has aborted with allocator
  corruption during exit. A separate graphics-cleanup crash has also been
  reproduced. The window can close normally while its child process dumps core.
  No rendering workaround is enabled by default; see
  [#5](https://github.com/The-FKing-Projects/Wingman/issues/5).
- Window-state restoration, a native About surface, custom branding,
  notifications, and tray integration are not implemented. External-link,
  popup, and offline-recovery behavior still needs characterization.

When reporting a problem, include the package/version, distribution, desktop
session (Wayland/X11), native dependency versions, and reproduction steps.
Distinguish closing the window from a crash while using the app. **Do not post
cookies, tokens, private conversations, or raw core dumps**, which can contain
account data.

## Build from source

Use Node.js 24 LTS, npm, a current stable Rust toolchain supporting edition 2024,
and Python 3.11+ for the metadata checks. Ubuntu 24.04 x86_64 is the automated
build baseline, not a promise of runtime support on other distributions.

On Ubuntu 24.04, install the native build and packaging prerequisites:

```sh
sudo apt update
sudo apt install build-essential curl wget file libxdo-dev libssl-dev libdbus-1-dev \
  libwebkit2gtk-4.1-dev librsvg2-dev patchelf squashfs-tools \
  desktop-file-utils rpm libarchive-tools
```

Install Node.js and Rust separately using their official installation guidance.
Other distributions need their equivalent packages; consult the
[Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux).

```sh
npm ci
python3 scripts/check_versions.py
python3 -m unittest discover -s scripts -p 'test_*.py'
npm run build
cargo check --locked --manifest-path src-tauri/Cargo.toml

# Interactive development; requires a graphical desktop session.
npm run tauri dev

# Build all Linux packages using the committed Rust dependency lockfile.
npm run tauri build -- --bundles deb rpm appimage -- --locked
```

Bundles are written under `src-tauri/target/release/bundle/`. Successful builds
do not establish installation, authentication, or graphics stability.

## Package verification

The packaging workflow verifies checksums, package identity/version/architecture,
DEB/RPM GTK and WebKit dependency declarations, launcher syntax and command, icons,
and x86_64 ELF executables inside each format. It extracts the AppImage without
starting Wingman. It also installs and removes the DEB on the disposable Ubuntu
24.04 runner and checks that its executable has no unresolved native libraries.
These checks do **not** launch a graphical session or validate Microsoft login.

To repeat the static checks on downloaded workflow artifacts, install the build
prerequisites above and run from the source checkout:

```sh
bash scripts/check_packages.sh /path/to/downloaded/artifacts 0.2.1
```

Only run this on trusted Wingman artifacts: AppImage inspection executes the
artifact's extraction runtime, and DEB installation can execute package scripts.
Checksums detect accidental corruption, not an untrusted source.

RPM payload inspection uses libarchive's `bsdtar`. The generated RPM omits the
archive-size header expected by Ubuntu 24.04's `rpm2cpio`, which can return a
failure after writing the payload. This tool-compatibility limitation does not
establish whether installation works on an RPM-based desktop; that still needs
separate testing.

| Environment | Evidence | Not established |
| --- | --- | --- |
| Ubuntu 24.04 x86_64 CI | DEB/RPM/AppImage builds; automated payload checks and DEB install/library-resolution checks | GUI startup, login, graphics stability |
| Ubuntu 26.04.1 maintainer desktop | Installed alpha DEB launches, signs in, and loads conversations | Reliable restart sign-in and crash-free shutdown |
| RPM-based desktops | RPM build and payload inspection only | Clean installation, GUI behavior |
| AppImage on desktop | Build and extracted payload inspection only | FUSE availability, GUI behavior, host compatibility |

For each release candidate, record its tag, SHA-256, distribution version,
Wayland/X11 session, and native WebKitGTK/GTK versions. Check clean installation,
application-menu entry/icon, startup, sign-in, conversation loading, normal
close/reopen, upgrade with profile retention, and uninstall. Record failures
explicitly and keep private account data out of reports. Retain the profile when
testing uninstall/upgrade unless testing deliberate data removal separately.

## Versions and releases

`src-tauri/tauri.conf.json` is the canonical application version. Keep
`package.json`, both root entries in `package-lock.json`, the Rust package version,
and its `Cargo.lock` entry in agreement. Each release gets a new numeric package
version; never replace the assets of an already published version.

Release tags use `vMAJOR.MINOR.PATCH`, optionally followed by `-alpha`, `-beta`,
or `-rc` and an optional numeric suffix (for example `v0.3.0-alpha.1`). The numeric
part must match every package version. Package versions currently omit the tag's
prerelease suffix; the GitHub release and documentation identify alpha/beta/rc
status.

Pull requests and pushes to `main` run metadata tests, the frontend build, and the
locked Rust check and tests. Pull requests also build all three package formats.
The **Linux packages** workflow can be run manually from the Actions tab to
rehearse packaging. Pull-request and manual runs upload workflow artifacts only
and cannot publish a release.

For a release:

1. Update all version metadata together and merge the source changes after checks
   pass. Run the packaging workflow manually for that commit.
2. Download its artifacts, verify `sha256sum -c SHA256SUMS`, and install/test the
   intended packages. Record the distro, dependency versions, artifact hashes,
   startup, login, normal close/reopen, and profile retention. Do not equate
   build success with runtime acceptance.
3. Create and push a matching version tag on the accepted commit. Tag builds
   publish packages and checksums to a **draft** GitHub release; tags with
   prerelease suffixes retain GitHub prerelease status.
4. Verify the draft's checksums and test the actual release artifacts before
   publishing it with an explicit maintainer or authorized agent action.
   Workflow artifacts and the draft are separate builds,
   so their hashes may differ. Add installation results and known limitations
   to the release notes.

Publishing refuses to overwrite an existing release or its assets. To retry a
failed packaging build, rerun it before a release exists. If publication partially
failed after creating a draft, inspect that draft and its assets before taking
manual recovery action; do not overwrite a published version.

## Roadmap

See the [milestones](https://github.com/The-FKing-Projects/Wingman/milestones):
v0.3.0 focuses on reliability and release engineering; v0.4.0 tentatively covers
desktop integration. Notifications, tray support, Flatpak, and Snap are
unscheduled proposals, not promised features.

## Disclaimer

Wingman is an independent open-source project and is not affiliated with, endorsed by, or sponsored by Microsoft Corporation or GitHub.

Microsoft Copilot and related trademarks are the property of their respective owners.

## License

MIT License