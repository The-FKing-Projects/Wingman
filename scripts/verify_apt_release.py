import argparse
import hashlib
from pathlib import Path
import re
import subprocess
import sys

from check_versions import TAG


def verify_apt_release(directory, tag):
    match = TAG.fullmatch(tag)
    if match is None:
        raise ValueError("Invalid Wingman release tag")
    version = match["version"]
    filename = f"Wingman_{version}_amd64.deb"
    expected_files = {filename, "SHA256SUMS"}
    if {path.name for path in directory.iterdir()} != expected_files:
        raise ValueError("Expected exactly the release's amd64 DEB and SHA256SUMS")
    for name in expected_files:
        path = directory / name
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"Expected a regular file: {name}")

    checksums = []
    for line in (directory / "SHA256SUMS").read_text().splitlines():
        entry = re.fullmatch(r"([0-9a-f]{64}) [ *](.+)", line)
        if entry is None:
            raise ValueError("Malformed SHA256SUMS entry")
        if entry[2] == filename:
            checksums.append(entry[1])
    if len(checksums) != 1:
        raise ValueError("Expected exactly one checksum for the DEB")
    package = directory / filename
    with package.open("rb") as stream:
        actual = hashlib.file_digest(stream, "sha256").hexdigest()
    if actual != checksums[0]:
        raise ValueError("DEB checksum does not match the published SHA256SUMS")

    result = subprocess.run(
        [
            "dpkg-deb", "--show",
            "--showformat=${Package}\n${Version}\n${Architecture}\n",
            str(package),
        ],
        check=True, capture_output=True, text=True,
    )
    if result.stdout.splitlines() != ["wingman", version, "amd64"]:
        raise ValueError("DEB package name, version, or architecture is incorrect")

    # The server receives only the DEB, not the release's RPM or AppImage.
    (directory / "SHA256SUMS").write_text(f"{actual}  {filename}\n")
    return filename


def main():
    parser = argparse.ArgumentParser(description="Verify a published Wingman DEB")
    parser.add_argument("directory", type=Path)
    parser.add_argument("tag")
    args = parser.parse_args()
    try:
        filename = verify_apt_release(args.directory, args.tag)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"APT release verification failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        return 1
    print(f"Verified {filename}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
