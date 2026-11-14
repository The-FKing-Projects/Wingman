<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Wingman icon: two swept-wing aircraft in echelon formation">
</p>

<h1 align="center">Wingman</h1>

Wingman is a lightweight Linux desktop wrapper for Microsoft Copilot, built with
Tauri v2 and the system WebKitGTK browser engine. A single native window loads
`https://copilot.com/chat`; Microsoft provides the web interface and authentication.
An internet connection is required.

## Current release

[v0.2.3-alpha](https://github.com/The-FKing-Projects/Wingman/releases/tag/v0.2.3-alpha)
is an early testing release, not a stable release.

- Native single-window application and desktop launcher.
- Microsoft sign-in and existing conversation loading have been reported working.
- DEB, RPM, and AppImage packages are published for x86_64.
- Browser data is stored on disk, but reliable sign-in across restarts is **not**
  established.

0.2.2 replaced the Tauri starter icons with the Wingman echelon icon. 0.2.3
makes the window's Wayland app ID and X11 class match `Wingman.desktop`, so KDE
Plasma shows the icon in the taskbar and title bar as well as the application
menu. Application behavior is otherwise unchanged from 0.2.1, so the 0.2.1 test
results below still describe the runtime. After upgrading, Plasma may keep a
generic taskbar icon until its cache refreshes; restarting the panel with
`systemctl --user restart plasma-plasmashell` or logging out fixes it.

The 0.2.1 patch release added automated builds and package verification, not
changes to the application's browsing or authentication behavior. The previous 0.2.0 DEB
was tested by the maintainer on Ubuntu 26.04.1; 0.2.1 has automated DEB installation
and native-library checks on Ubuntu 24.04. RPM and AppImage payloads are inspected.
The published 0.2.1 RPM also passed installation, native-library resolution, and
removal checks in a disposable Fedora 44 container; desktop runtime behavior
was also smoke-tested in a Fedora 45 GNOME/Wayland QEMU VM, including a test
conversation and retained login after app restart, with no crashes reported.
The same RPM installed and launched in an openSUSE Leap 16.1 KDE/Wayland QEMU VM,
where login also survived close/reopen; zypper required an exception for the
unsigned package.
The AppImage launched and signed in on the Ubuntu 26.04.1 host, but showed the
same shutdown crash and restart sign-in failure as the DEB there and reports a
missing GStreamer `appsink` element.
There is not yet a tested cross-distribution support matrix. Packages depend on
their host's native libraries; an AppImage does not guarantee compatibility with
every Linux distribution or graphics driver.

## Install

Download the package for your system from the release link above. For the existing
alpha assets:

```sh
# Debian/Ubuntu: run from the download directory.
sudo apt install ./Wingman_0.2.3_amd64.deb

# Fedora/RPM systems: smoke-tested in a Fedora 45 GNOME/Wayland VM.
sudo dnf install ./Wingman-0.2.3-1.x86_64.rpm

# AppImage: launches on Ubuntu 26.04.1; see known issues below.
chmod +x Wingman_0.2.3_amd64.AppImage
./Wingman_0.2.3_amd64.AppImage
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
  session. Login survived app restarts in the Fedora 45 and openSUSE Leap 16.1
  VM smoke tests, but not on the tested Ubuntu 26.04.1 host. The cause remains
  under investigation in
  [#8](https://github.com/The-FKing-Projects/Wingman/issues/8).
- **Shutdown crashes:** a WebKit child process has aborted with allocator
  corruption during exit. A separate graphics-cleanup crash has also been
  reproduced. The window can close normally while its child process dumps core.
  No rendering workaround is enabled by default; see
  [#5](https://github.com/The-FKing-Projects/Wingman/issues/5).
- Window-state restoration, a native About surface, notifications, and tray
  integration are not implemented. External-link, popup, and offline-recovery
  behavior still needs characterization.

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
bash scripts/check_packages.sh /path/to/downloaded/artifacts 0.2.3
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
| Ubuntu 26.04.1 x86_64 / Wayland, v0.2.0 DEB (prior maintainer test) | Installed DEB launches, signs in, and loads conversations | Reliable restart sign-in and crash-free shutdown |
| Ubuntu 26.04.1 x86_64 / Wayland, v0.2.1 DEB (current installed package) | Installed executable matches the published DEB payload; maintainer reports normal visible open/close, but a WebKit child aborts after close and sign-in is required again on restart; WebKitGTK 2.52.6, GTK 3.24.52, GLib 2.88.0 | Crash-free shutdown and restart sign-in fail; upgrade/profile retention and conversation-loading acceptance remain unverified |
| Fedora 44 x86_64 container, published v0.2.1 RPM | Checksum, DNF installation, RPM file integrity, native-library resolution, launcher/icon presence, and package removal pass; WebKitGTK 2.54.2, GTK 3.24.52, GLib 2.88.4 | No GUI launched; desktop integration, login, rendering, close/reopen, upgrade/profile retention, and shutdown behavior |
| Fedora 45 x86_64 QEMU VM, GNOME / Wayland, published v0.2.1 RPM | Release SHA-256 matches; DNF installation, startup, test conversation, and retained login after app restart reported successful; no crashes reported; WebKitGTK 2.54.2, GTK 3.24.52, GLib 2.90.1 | Child-core accounting, long-session stability, launcher/icon appearance, existing-conversation loading, upgrade/profile retention, and uninstall |
| openSUSE Leap 16.1 x86_64 QEMU VM, KDE / Wayland, published v0.2.1 RPM | Release SHA-256 matches; zypper dependency resolution and installation succeeded after accepting the unsigned-package warning; startup and retained login after close/reopen reported successful; WebKitGTK 2.54.1, GTK 3.24.52, GLib 2.84.4 | Package signature verification, child-core accounting, long-session stability, launcher/icon appearance, conversation-loading acceptance, upgrade/profile retention, and uninstall |
| Ubuntu 26.04.1 x86_64 / Wayland, published v0.2.1 AppImage | Release SHA-256 matches; FUSE startup, sign-in, and Copilot use reported successful | Background WebKit child abort after closing a longer session; sign-in not retained (shares the DEB's profile); GStreamer `appsink` missing; desktop-entry integration and other hosts untested |

A single maintainer-controlled v0.2.1 launch with
`WEBKIT_DISABLE_DMABUF_RENDERER=1` rendered and worked normally, then closed
without a reported error or a recorded WebKit child core in the observation
interval. A follow-up batch of three baseline and three override launches also
produced no reported crashes or recorded WebKit child cores; the first baseline
was interrupted with Ctrl+C and is not a normal-close result. These short trials
do not establish a renderer-specific improvement or a verified fix; shutdown
failures are intermittent and the default rendering policy remains unchanged.

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

## APT repository publishing

The **Publish APT repository** workflow imports the DEB from each newly published
GitHub release into the existing APTly repository at
<https://repo.fkingprojects.org>. **Prereleases are included**, even though the
APT distribution is named `stable`. Draft releases, pull requests, and tag builds
alone do not update the APT repository. The existing build, verification, and
draft-release approval process is unchanged.

A GitHub-hosted job downloads the release's DEB and `SHA256SUMS`, checks the hash
and package name/version/architecture, and passes only the verified DEB and its
checksum to the server. It does not rebuild or install the package. The publishing
job does not check out repository code or execute package contents.

The existing organization runner `FK04-SERVER` must be available to Wingman with
labels `self-hosted`, `Linux`, `X64`, and `fkingprojects`. The custom label keeps
publishing on the APT-capable organization runner. Do not allow untrusted
workflows or pull-request jobs to use this signing-capable server.

The runner's systemd service account, `github-runner`, must have `aptly`, Bash,
and `sha256sum` on its service PATH and use the same HOME, APTly configuration,
GnuPG home, filesystem permissions, and signing-key access as the validated
interactive setup. Signing must work noninteractively: the workflow uses
`-batch`, never `sudo`, never skips signing, and does not import a private key.
There are no new repository secrets to configure. The commands are:

```sh
aptly repo add fkingprojects /path/to/verified/Wingman_VERSION_amd64.deb
aptly publish update -batch stable .
```

This preserves the existing prefix `.`, distribution `stable`, component `main`,
and architectures `all amd64`. It assumes the existing web server serves APTly's
published files at the repository URL; it does not change APTly or web-server
configuration. Workflow runs are serialized to avoid concurrent Wingman updates;
other publishers on this server must coordinate their own APTly writes.

After this workflow is on the default branch, publish a GitHub release normally,
or run **Publish APT repository** manually in Actions with an already published
tag (for example `v0.2.1-alpha`). Manual runs reject drafts and nonexistent
releases. Use the same manual trigger or rerun a failed workflow to retry; imports
do not force-replace packages with conflicting contents. If import succeeds but
signing/publication fails, the package remains in the local APTly repository and
the workflow fails visibly; correct signing/access before retrying.

Package versions still omit prerelease suffixes, so do not publish distinct
alpha/beta/stable builds with the same numeric version. APT cannot distinguish
them, and conflicting contents must not be overwritten. Older package versions
are retained; replaying an older release does not downgrade APT's newest version.
Repository clients must already trust the repository's public signing key and
have its `stable main` source configured; signing-key distribution is unchanged.

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