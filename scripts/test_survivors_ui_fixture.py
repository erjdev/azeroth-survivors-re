"""Isolation checks; the Rust account tests validate the fixture save contents."""
import importlib.util
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest
from unittest.mock import patch

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

    def test_preset_is_copied_without_mutating_reference_or_overwriting_request(self):
        with tempfile.TemporaryDirectory() as temp:
            home = fixture.prepare("partial", Path(temp) / "preview with spaces")
            source = fixture.PREVIEWS / "talents-640.txt"
            original = source.read_bytes()
            request = fixture.preview(home, source.name)
            self.assertEqual(request.read_bytes(), original)
            request.write_text("page=collection")
            self.assertEqual(source.read_bytes(), original)
            with self.assertRaises(FileExistsError):
                fixture.preview(home, source.name)
            for name in ("../talents-640.txt", str(source), "missing.txt"):
                with self.assertRaises(ValueError):
                    fixture.preview(home, name)

    def test_cli_prepares_preset_and_rejects_unknown_before_creating_state(self):
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp) / "preview with spaces"
            command = [sys.executable, str(Path(fixture.__file__)), "partial",
                       "--output", str(home), "--preset", "talents-640.txt"]
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(str(home / "view.txt"), result.stdout)
            self.assertEqual((home / "view.txt").read_bytes(),
                             (fixture.PREVIEWS / "talents-640.txt").read_bytes())
            missing = Path(temp) / "must not exist"
            command[-3:] = [str(missing), "--preset", "missing.txt"]
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(missing.exists())

    def test_saved_spec_changes_only_selected_fixture_hero_and_preserves_source(self):
        original = (fixture.FIXTURES / "trained.txt").read_bytes()
        with tempfile.TemporaryDirectory() as temp:
            for selected in (1, 2, 3):
                home = fixture.prepare("trained", Path(temp) / str(selected), 2, selected)
                actual = (home / "benilla-config/survivors-account.txt").read_bytes()
                expected = original.replace(b"who 2 5 3000 Nara Restoration 0|||",
                                            f"who 2 5 3000 Nara Restoration {selected - 1}|||".encode())
                self.assertEqual(actual, expected)
                request = fixture.preview(home, "talents-640.txt", 2)
                self.assertIn("hero=2", request.read_text())
                self.assertEqual(request.read_text().count("hero="), 1)
            self.assertEqual((fixture.FIXTURES / "trained.txt").read_bytes(), original)

    def test_invalid_hero_or_spec_leaves_destination_absent(self):
        with tempfile.TemporaryDirectory() as temp:
            for hero, selected in [(999, 1), (0, 1), (2, 0), (2, 4), (None, 1), (2, None)]:
                home = Path(temp) / "unused"
                with self.assertRaises(ValueError):
                    fixture.prepare("trained", home, hero, selected)
                self.assertFalse(home.exists())

    def test_spec_cli_targets_hero_and_requires_paired_options(self):
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp) / "spec"
            base = [sys.executable, str(Path(fixture.__file__)), "trained", "--output", str(home)]
            invalid = subprocess.run(base + ["--spec", "2"], capture_output=True, text=True)
            self.assertNotEqual(invalid.returncode, 0)
            self.assertFalse(home.exists())
            result = subprocess.run(base + ["--hero", "2", "--spec", "2", "--preset", "talents-640.txt"], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("hero=2", (home / "view.txt").read_text())
            self.assertIn("who 2 5 3000 Nara Restoration 1|||", (home / "benilla-config/survivors-account.txt").read_text())

    def test_launch_is_isolated_shell_free_and_propagates_failure(self):
        with tempfile.TemporaryDirectory() as temp:
            home = fixture.prepare("partial", Path(temp) / "preview with spaces")
            request = fixture.preview(home, "talents-640.txt")
            inherited = {"BENILLA_HOME": "player-save", "WOW_DATA": "other-install",
                         "WOW_CAPTURE": "1", "WOW_SURVIVORS_AUTO": "1",
                         "WOW_SURVIVORS_UI_PREVIEW": "other-preview",
                         "WOW_SURVIVORS_UI_PAGE": "other-page", "PATH": "toolchain"}
            with patch.dict(fixture.os.environ, inherited, clear=True):
                with patch.object(fixture.subprocess, "run") as run:
                    run.return_value.returncode = 7
                    self.assertEqual(fixture.launch(home, "talents", request), 7)
                    args, kwargs = run.call_args
                    self.assertEqual(args, (["cargo", "survivors"],))
                    self.assertEqual(kwargs["cwd"], fixture.ROOT)
                    self.assertFalse(kwargs.get("shell", False))
                    env = kwargs["env"]
                    self.assertEqual(env["BENILLA_HOME"], str(home))
                    self.assertEqual(env["WOW_DATA"], str(fixture.ROOT / "wow-data/Data"))
                    self.assertEqual(env["WOW_SURVIVORS_UI_PREVIEW"], str(request))
                    self.assertEqual(env["PATH"], "toolchain")
                    for key in ("WOW_CAPTURE", "WOW_SURVIVORS_AUTO", "WOW_SURVIVORS_UI_PAGE"):
                        self.assertNotIn(key, env)
                env = fixture.launch_environment(home, "talents")
                self.assertEqual(env["WOW_SURVIVORS_UI_PAGE"], "talents")
                self.assertNotIn("WOW_SURVIVORS_UI_PREVIEW", env)
                self.assertEqual(dict(fixture.os.environ), inherited)


if __name__ == "__main__":
    unittest.main()
