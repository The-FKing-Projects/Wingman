import argparse
import json
from pathlib import Path
import re
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]
VERSION = r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
TAG = re.compile(
    rf"v(?P<version>{VERSION})(?:-(?P<stage>alpha|beta|rc)"
    r"(?:\.(?:0|[1-9][0-9]*))?)?"
)


def check_versions(root, tag=None):
    config = json.loads((root / "src-tauri/tauri.conf.json").read_text())
    package = json.loads((root / "package.json").read_text())
    npm_lock = json.loads((root / "package-lock.json").read_text())
    cargo = tomllib.loads((root / "src-tauri/Cargo.toml").read_text())
    cargo_lock = tomllib.loads((root / "src-tauri/Cargo.lock").read_text())
    expected = config["version"]
    if not isinstance(expected, str) or not re.fullmatch(VERSION, expected):
        raise ValueError("Tauri version must be a numeric MAJOR.MINOR.PATCH")
    root_packages = [
        entry for entry in cargo_lock["package"]
        if entry["name"] == cargo["package"]["name"] and "source" not in entry
    ]
    if len(root_packages) != 1:
        raise ValueError("Cargo.lock must contain exactly one local app package")
    versions = {
        "package.json": package["version"],
        "package-lock.json": npm_lock["version"],
        "package-lock.json packages['']": npm_lock["packages"][""]["version"],
        "Cargo.toml": cargo["package"]["version"],
        "Cargo.lock": root_packages[0]["version"],
    }
    for location, actual in versions.items():
        if actual != expected:
            raise ValueError(f"{location}: expected {expected}, found {actual}")
    if tag is not None:
        match = TAG.fullmatch(tag)
        if match is None:
            raise ValueError("Invalid release tag; expected vX.Y.Z[-alpha|beta|rc[.N]]")
        if match["version"] != expected:
            raise ValueError(f"Tag {tag} does not match app version {expected}")
    return expected


def main():
    parser = argparse.ArgumentParser(description="Check all Wingman package versions")
    parser.add_argument("--tag", help="Also validate a release tag")
    args = parser.parse_args()
    try:
        version = check_versions(ROOT, args.tag)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Version check failed: {error}", file=sys.stderr)
        return 1
    print(f"All application versions match {version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
