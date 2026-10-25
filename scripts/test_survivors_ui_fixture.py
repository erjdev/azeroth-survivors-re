"""Isolation checks; the Rust account tests validate the fixture save contents."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    "fixture", Path(__file__).with_name("survivors-ui-fixture.py")
)
fixture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixture)


class FixtureTests(unittest.TestCase):
    def test_copies_each_profile_and_refuses_existing_state(self):
        with tempfile.TemporaryDirectory() as temp:
            for profile in fixture.PROFILES:
                home = fixture.prepare(profile, Path(temp) / profile)
                saved = home / "benilla-config/survivors-account.txt"
                self.assertEqual(saved.read_bytes(), (fixture.FIXTURES / f"{profile}.txt").read_bytes())
                saved.write_text("player edited this preview")
                with self.assertRaises(FileExistsError):
                    fixture.prepare(profile, home)
                self.assertEqual(saved.read_text(), "player edited this preview")

    def test_refuses_symlink_destination_and_unknown_profile(self):
        with tempfile.TemporaryDirectory() as temp:
            target = Path(temp) / "existing"
            target.mkdir()
            link = Path(temp) / "link"
            link.symlink_to(target, target_is_directory=True)
            with self.assertRaises(FileExistsError):
                fixture.prepare("fresh", link)
            with self.assertRaises(ValueError):
                fixture.prepare("unknown", Path(temp) / "new")
            self.assertFalse((Path(temp) / "new").exists())
            self.assertEqual(list(target.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
