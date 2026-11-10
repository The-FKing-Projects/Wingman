import json
from pathlib import Path
import tempfile
import unittest

from check_versions import check_versions


class VersionTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "src-tauri").mkdir()
        self.write_json("src-tauri/tauri.conf.json", {"version": "0.2.0"})
        self.write_json("package.json", {"version": "0.2.0"})
        self.write_json("package-lock.json", {
            "version": "0.2.0", "packages": {"": {"version": "0.2.0"}},
        })
        (self.root / "src-tauri/Cargo.toml").write_text(
            '[package]\nname = "wingman"\nversion = "0.2.0"\n'
        )
        (self.root / "src-tauri/Cargo.lock").write_text(
            '[[package]]\nname = "wingman"\nversion = "0.2.0"\n'
        )

    def write_json(self, name, value):
        (self.root / name).write_text(json.dumps(value))

    def test_matching_versions(self):
        self.assertEqual(check_versions(self.root), "0.2.0")

    def test_valid_tags(self):
        for tag in ("v0.2.0", "v0.2.0-alpha", "v0.2.0-beta.2", "v0.2.0-rc.1"):
            with self.subTest(tag=tag):
                self.assertEqual(check_versions(self.root, tag), "0.2.0")

    def test_invalid_or_mismatched_tags(self):
        for tag in (
            "", "0.2.0", "v0.3.0", "v0.2.0-other", "v0.2.0-alpha.01",
            "v00.2.0", "v0.2.0\n", "v0.2.0;echo bad",
        ):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                check_versions(self.root, tag)

    def test_each_json_version_is_checked(self):
        for filename, value in (
            ("package.json", {"version": "0.1.0"}),
            ("package-lock.json", {
                "version": "0.1.0", "packages": {"": {"version": "0.2.0"}},
            }),
            ("package-lock.json", {
                "version": "0.2.0", "packages": {"": {"version": "0.1.0"}},
            }),
        ):
            with self.subTest(filename=filename):
                original = (self.root / filename).read_text()
                self.write_json(filename, value)
                with self.assertRaises(ValueError):
                    check_versions(self.root)
                (self.root / filename).write_text(original)

    def test_each_rust_version_is_checked(self):
        for name in ("Cargo.toml", "Cargo.lock"):
            with self.subTest(name=name):
                path = self.root / "src-tauri" / name
                original = path.read_text()
                path.write_text(original.replace("0.2.0", "0.1.0"))
                with self.assertRaises(ValueError):
                    check_versions(self.root)
                path.write_text(original)

    def test_missing_local_rust_package(self):
        (self.root / "src-tauri/Cargo.lock").write_text(
            '[[package]]\nname = "other"\nversion = "0.2.0"\n'
        )
        with self.assertRaises(ValueError):
            check_versions(self.root)

    def test_non_numeric_canonical_version(self):
        self.write_json("src-tauri/tauri.conf.json", {"version": "0.2.0-alpha"})
        with self.assertRaises(ValueError):
            check_versions(self.root)


if __name__ == "__main__":
    unittest.main()
