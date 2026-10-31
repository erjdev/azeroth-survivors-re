"""Hermetic check-scope fixtures: no real compiler, install or network."""
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

class CheckBaseline(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='survivors-check-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.env = {**os.environ, 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'}
        for name in ['CHECK_FULL', 'GATES_FORCE', 'FAIL_RESOLVE']:
            self.env.pop(name, None)
        for directory in ['scripts', 'bin', 'target', 'crates/alpha/src', 'crates/beta/src', 'web', 'docs']:
            (self.root / directory).mkdir(parents=True)
        shutil.copy(Path(__file__).with_name('check.sh'), self.root / 'scripts/check.sh')
        gate = self.root / 'scripts/gates.sh'
        gate.write_text('#!/bin/bash\necho FULL_GATE\n')
        gate.chmod(0o755)
        (self.root / '.gitignore').write_text('target/\nbin/\ncalls.txt\n')
        (self.root / 'crates/alpha/src/lib.rs').write_text('// original\n')
        (self.root / 'crates/beta/src/lib.rs').write_text('// dependent\n')
        (self.root / 'web/boot.js').write_text('// original\n')
        rust = self.root / 'bin/rustc'
        rust.write_text('#!/bin/bash\necho "rustc fixture"\n')
        rust.chmod(0o755)
        cargo = self.root / 'bin/cargo'
        cargo.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
root = pathlib.Path.cwd()
with (root / 'calls.txt').open('a') as f: f.write(' '.join(args) + '\\n')
if args[0] == 'run':
    if os.getenv('FAIL_RESOLVE'): sys.exit(1)
    print('/fixture/data')
elif args[0] == 'metadata':
    print(json.dumps({'packages': [
        {'name': name, 'manifest_path': str(root / 'crates' / name / 'Cargo.toml'),
         'dependencies': ([{'name': 'alpha'}] if name == 'beta' else [])}
        for name in ['alpha', 'beta']]}))
''')
        cargo.chmod(0o755)
        self.env['PATH'] = str(self.root / 'bin') + os.pathsep + self.env['PATH']
        self.git('init', '-q', '-b', 'main')
        self.commit('base')
        self.git('checkout', '-q', '-b', 'feature')
        (self.root / 'web/boot.js').write_text('// already checked branch change\n')
        self.commit('old web work')
        self.tree = self.git('rev-parse', 'HEAD^{tree}').strip()
        self.stamp = self.root / 'target/.gates-green'
        self.valid = f'{self.tree}|rustc fixture|{hashlib.sha1(gate.read_bytes()).hexdigest()}|/fixture/data\n'
        self.stamp.write_text(self.valid)

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.root, env=self.env, text=True)

    def commit(self, title):
        self.git('add', '-A')
        self.git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
                 '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', title)

    def run_check(self, **env):
        result = subprocess.run(['bash', 'scripts/check.sh'], cwd=self.root,
                                env={**self.env, **env}, text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result.stdout

    def assert_scoped(self):
        output = self.run_check()
        self.assertIn('check: scope = alpha beta', output)
        self.assertNotIn('FULL_GATE', output)
        self.assertIn('test -p alpha -p beta', (self.root / 'calls.txt').read_text())

    def test_old_web_work_does_not_force_full_for_new_crate_edit(self):
        (self.root / 'crates/alpha/src/lib.rs').write_text('// changed\n')
        self.assert_scoped()

    def test_committed_followup_is_still_in_scope(self):
        (self.root / 'crates/alpha/src/lib.rs').write_text('// changed\n')
        self.commit('followup')
        self.assert_scoped()

    def test_untracked_crate_file_is_checked(self):
        (self.root / 'crates/alpha/src/new.rs').write_text('// new\n')
        self.assert_scoped()

    def test_deleted_crate_file_is_checked(self):
        (self.root / 'crates/alpha/src/lib.rs').unlink()
        self.assert_scoped()

    def test_staged_change_is_not_hidden_by_unstaged_revert(self):
        source = self.root / 'crates/alpha/src/lib.rs'
        original = source.read_text()
        source.write_text('// staged change\n')
        self.git('add', str(source))
        source.write_text(original)
        self.assert_scoped()

    def test_reverting_verified_web_change_is_a_new_change(self):
        (self.root / 'web/boot.js').write_text('// original\n')
        self.assertIn('FULL_GATE', self.run_check())

    def test_new_web_change_still_forces_full(self):
        (self.root / 'web/boot.js').write_text('// new unverified web change\n')
        self.assertIn('FULL_GATE', self.run_check())

    def test_docs_only_runs_formatting(self):
        (self.root / 'docs/progress.md').write_text('Progress\n')
        output = self.run_check()
        self.assertIn('docs-only round', output)
        self.assertNotIn('FULL_GATE', output)

    def test_bad_metadata_falls_back_to_branch_comparison(self):
        variants = ['', self.valid.replace('rustc fixture', 'rustc other'),
                    self.valid.replace('/fixture/data', '/different/data'),
                    self.valid.replace(self.tree, '0' * 40), self.valid.replace(self.tree, 'HEAD'),
                    self.valid.replace(self.tree, self.tree + '^{tree}'), 'malformed\n']
        for stamp in variants:
            with self.subTest(stamp=stamp):
                self.stamp.write_text(stamp)
                self.assertIn('FULL_GATE', self.run_check())

    def test_missing_stamp_falls_back(self):
        self.stamp.unlink()
        self.assertIn('FULL_GATE', self.run_check())

    def test_gate_script_change_invalidates_baseline(self):
        with (self.root / 'scripts/gates.sh').open('a') as f: f.write('# changed\n')
        self.assertIn('FULL_GATE', self.run_check())

    def test_corpus_change_invalidates_baseline(self):
        (self.root / 'wow-addons-vanilla').mkdir()
        self.assertIn('FULL_GATE', self.run_check())

    def test_data_resolver_failure_falls_back(self):
        self.assertIn('FULL_GATE', self.run_check(FAIL_RESOLVE='1'))

    def test_full_override_ignores_baseline(self):
        self.assertIn('FULL_GATE', self.run_check(CHECK_FULL='1'))
        self.assertIn('FULL_GATE', self.run_check(GATES_FORCE='1'))

class GateStamp(unittest.TestCase):
    setUp = CheckBaseline.setUp
    git = CheckBaseline.git
    commit = CheckBaseline.commit
    # Run only stamp-specific tests here; scope tests use the stub gate above.
    def run_stamp_case(self, change):
        shutil.copy(Path(__file__).with_name('gates.sh'), self.root / 'scripts/gates.sh')
        (self.root / 'scripts/gates.sh').chmod(0o755)
        for name in ['doc-links.py', 'pass-span-lint.py']:
            path = self.root / 'scripts' / name
            path.write_text('#!/bin/bash\nexit 0\n')
            path.chmod(0o755)
        cargo = self.root / 'bin/cargo'
        with cargo.open('a') as f:
            f.write("\nif args[0] == 'test':\n    (root / os.environ['MUTATE_PATH']).write_text('// changed during gate\\n')\n")
        self.commit('gate fixture')
        index_before = self.git('write-tree').strip()
        stamp_before = self.stamp.read_text()
        result = subprocess.run(['bash', 'scripts/gates.sh'], cwd=self.root,
            env={**self.env, 'GATES_FORCE': '1', 'MUTATE_PATH': change}, text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('ALL GATES GREEN', result.stdout)
        self.assertEqual(self.git('write-tree').strip(), index_before, 'real index changed')
        return stamp_before

    def test_docs_only_progress_can_stamp_the_verified_source(self):
        before = self.run_stamp_case('docs/progress.md')
        self.assertNotEqual(self.stamp.read_text(), before)
        tree = self.stamp.read_text().split('|')[0]
        self.assertIn('docs/progress.md', self.git('ls-tree', '-r', '--name-only', tree))

    def test_source_edit_during_run_cannot_stamp_unverified_source(self):
        before = self.run_stamp_case('crates/alpha/src/lib.rs')
        self.assertEqual(self.stamp.read_text(), before)

if __name__ == '__main__':
    unittest.main()
