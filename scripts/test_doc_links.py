"""The documentation gate must reject incomplete rustdoc runs, not just dead links."""
import contextlib
import importlib.util
import io
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    'doc_links', Path(__file__).with_name('doc-links.py'))
doc_links = importlib.util.module_from_spec(spec)
spec.loader.exec_module(doc_links)


class DocLinksTests(unittest.TestCase):
    def run_gate(self, result=None, error=None, names=()):
        output = io.StringIO()
        with patch.object(doc_links.subprocess, 'run', return_value=result, side_effect=error), \
                patch.object(doc_links, 'declared_names', return_value=set(names)) as scan, \
                patch.object(doc_links.sys, 'argv', ['doc-links.py']), \
                contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
            status = doc_links.main()
        return status, output.getvalue(), scan.called

    def test_compile_failure_without_link_warnings_is_not_success(self):
        result = subprocess.CompletedProcess(['cargo'], 101, 'build output', 'error: failed to compile')
        status, output, scanned = self.run_gate(result)
        self.assertEqual(status, 1)
        self.assertIn('exited with 101', output)
        self.assertIn('failed to compile', output)
        self.assertIn('build output', output)
        self.assertNotIn('doc-links ok', output)
        self.assertFalse(scanned)

    def test_missing_cargo_is_a_reported_failure(self):
        status, output, scanned = self.run_gate(error=FileNotFoundError('cargo unavailable'))
        self.assertEqual(status, 1)
        self.assertIn('could not run cargo doc', output)
        self.assertFalse(scanned)

    def test_successful_rustdoc_still_rejects_deleted_targets(self):
        warning = 'warning: unresolved link to `crate::Removed`\n --> src/example.rs:12'
        result = subprocess.CompletedProcess(['cargo'], 0, '', warning)
        status, output, scanned = self.run_gate(result)
        self.assertEqual(status, 1)
        self.assertIn('crate::Removed', output)
        self.assertTrue(scanned)
        status, output, _ = self.run_gate(result, names=['Removed'])
        self.assertEqual(status, 0)
        self.assertIn('doc-links ok', output)


if __name__ == '__main__':
    unittest.main()
