"""Exercise native Windows wrappers without compiling or launching the game."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


@unittest.skipUnless(os.name == "nt", "requires Windows cmd and PowerShell")
class WindowsLaunchers(unittest.TestCase):
    def test_working_directory_data_selection_and_native_exit_status(self):
        source = Path(__file__).resolve().parent
        powershell = shutil.which("pwsh") or shutil.which("powershell")
        self.assertIsNotNone(powershell, "Windows launcher gate requires PowerShell")
        with tempfile.TemporaryDirectory(prefix="survivors launcher ") as temporary:
            root = Path(temporary) / "checkout with spaces"
            scripts = root / "scripts"
            scripts.mkdir(parents=True)
            for name in ("survivors.ps1", "survivors.cmd"):
                shutil.copy2(source / name, scripts / name)
            mock_bin = Path(temporary) / "mock tools"
            mock_bin.mkdir()
            (mock_bin / "cargo.cmd").write_text(
                '@echo off\n'
                '> "%LAUNCHER_TEST_TRACE%" echo %CD%\n'
                '>> "%LAUNCHER_TEST_TRACE%" echo [%WOW_DATA%]\n'
                '>> "%LAUNCHER_TEST_TRACE%" echo %*\n'
                'exit /b %LAUNCHER_TEST_EXIT%\n'
            )
            data = root / "wow-data" / "Data"
            data.mkdir(parents=True)
            marker = data / "terrain.MPQ"
            trace = Path(temporary) / "invocation.txt"
            for shell in ("powershell", "cmd"):
                for local_data, explicit_data in ((False, None), (True, None), (True, "D:\\External Data")):
                    for status in (0, 37):
                        with self.subTest(shell=shell, local_data=local_data,
                                          explicit_data=explicit_data, status=status):
                            if local_data:
                                marker.touch()
                            else:
                                marker.unlink(missing_ok=True)
                            trace.unlink(missing_ok=True)
                            env = os.environ.copy()
                            env.pop("WOW_DATA", None)
                            if explicit_data:
                                env["WOW_DATA"] = explicit_data
                            env["PATH"] = str(mock_bin) + os.pathsep + env["PATH"]
                            env["LAUNCHER_TEST_TRACE"] = str(trace)
                            env["LAUNCHER_TEST_EXIT"] = str(status)
                            command = (
                                [powershell, "-NoProfile",
                                 "-File", str(scripts / "survivors.ps1")]
                                if shell == "powershell" else
                                ["cmd.exe", "/d", "/c", str(scripts / "survivors.cmd")]
                            )
                            result = subprocess.run(command, cwd=temporary, env=env,
                                                    capture_output=True, text=True, timeout=30)
                            self.assertEqual(result.returncode, status, result.stdout + result.stderr)
                            self.assertTrue(trace.exists(), result.stdout + result.stderr)
                            lines = trace.read_text().splitlines()
                            expected_data = explicit_data or (str(data) if local_data else "")
                            self.assertEqual(lines, [str(root), f"[{expected_data}]", "survivors"])


if __name__ == "__main__":
    unittest.main()
