import hashlib
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from verify_apt_release import verify_apt_release


class AptReleaseTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.filename = "Wingman_0.2.1_amd64.deb"
        self.package = self.root / self.filename
        self.package.write_bytes(b"test package")
        self.digest = hashlib.sha256(self.package.read_bytes()).hexdigest()
        self.checksum = self.root / "SHA256SUMS"
        self.entry = f"{self.digest}  {self.filename}\n"
        self.checksum.write_text(
            self.entry + f"{'0' * 64}  Wingman_0.2.1_amd64.AppImage\n"
        )
        self.metadata = patch("verify_apt_release.subprocess.run")
        self.run = self.metadata.start()
        self.addCleanup(self.metadata.stop)
        self.run.return_value.stdout = "wingman\n0.2.1\namd64\n"

    def test_published_tags_and_deb_only_manifest(self):
        for tag in ("v0.2.1", "v0.2.1-alpha", "v0.2.1-beta.2", "v0.2.1-rc.1"):
            with self.subTest(tag=tag):
                self.assertEqual(verify_apt_release(self.root, tag), self.filename)
                self.assertEqual(self.checksum.read_text(), self.entry)

    def test_invalid_or_mismatched_tags(self):
        for tag in ("v0.2.2", "--help", "v0.2.1\n", "v0.2.1-alpha.01"):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                verify_apt_release(self.root, tag)
        self.run.assert_not_called()

    def test_missing_extra_or_renamed_package(self):
        self.package.rename(self.root / "other.deb")
        with self.assertRaises(ValueError):
            verify_apt_release(self.root, "v0.2.1")
        self.package.write_bytes(b"test package")
        with self.assertRaises(ValueError):
            verify_apt_release(self.root, "v0.2.1")
        self.run.assert_not_called()

    def test_rejects_symlink(self):
        self.package.unlink()
        self.package.symlink_to(self.checksum)
        with self.assertRaises(ValueError):
            verify_apt_release(self.root, "v0.2.1")
        self.run.assert_not_called()

    def test_rejects_changed_contents(self):
        self.package.write_bytes(b"corrupted")
        with self.assertRaises(ValueError):
            verify_apt_release(self.root, "v0.2.1")
        self.run.assert_not_called()

    def test_rejects_missing_duplicate_or_malformed_checksum(self):
        for contents in (
            f"{'0' * 64}  other.deb\n",
            self.entry * 2,
            f"not-a-checksum  {self.filename}\n",
            f"{self.digest}  ../{self.filename}\n",
        ):
            with self.subTest(contents=contents):
                self.checksum.write_text(contents)
                with self.assertRaises(ValueError):
                    verify_apt_release(self.root, "v0.2.1")
        self.run.assert_not_called()

    def test_rejects_incorrect_package_metadata(self):
        for output in (
            "other\n0.2.1\namd64\n",
            "wingman\n0.2.0\namd64\n",
            "wingman\n0.2.1\narm64\n",
        ):
            with self.subTest(output=output):
                self.run.return_value.stdout = output
                with self.assertRaises(ValueError):
                    verify_apt_release(self.root, "v0.2.1")

    def test_dpkg_failure_is_not_ignored(self):
        self.run.side_effect = subprocess.CalledProcessError(2, "dpkg-deb")
        with self.assertRaises(subprocess.CalledProcessError):
            verify_apt_release(self.root, "v0.2.1")


if __name__ == "__main__":
    unittest.main()
