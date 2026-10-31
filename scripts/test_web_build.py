"""Exercise build orchestration without compiling Rust or requiring wasm tooling."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
STUB = r'''
import json, os, pathlib, sys
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ['BUILD_TEST_LOG'], 'a') as log:
    log.write(json.dumps([name, *args]) + '\n')
version = os.environ['BUILD_TEST_VERSION']
if name == 'cargo':
    profile = args[args.index('--profile') + 1]
    path = pathlib.Path('target/wasm32-unknown-unknown') / profile / 'wenilla.wasm'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(version)
elif name == 'wasm-bindgen':
    if args == ['--version']:
        print('wasm-bindgen ' + os.environ.get('BUILD_TEST_BINDGEN_VERSION', '0.2.122'))
        sys.exit(0)
    source = next(pathlib.Path(a) for a in args if a.endswith('.wasm'))
    assert source.read_text() == version
    out = pathlib.Path(args[args.index('--out-dir') + 1])
    for filename in ['wenilla.js', 'wenilla_bg.wasm']:
        (out / filename).write_text(version)
elif name == 'wasm-opt':
    source = next(pathlib.Path(a) for a in args if a.endswith('.wasm'))
    pathlib.Path(args[args.index('-o') + 1]).write_text(source.read_text() + '-optimized')
elif name == 'brotli':
    source = pathlib.Path(args[args.index('-o') - 1])
    pathlib.Path(args[args.index('-o') + 1]).write_text(source.read_text())
elif name == 'gzip':
    source = pathlib.Path(args[-1])
    pathlib.Path(str(source) + '.gz').write_text(source.read_text())
'''


class WebBuildTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'Cargo.lock').write_text('[[package]]\nname = \"wasm-bindgen\"\nversion = \"0.2.122\"\n')
        (self.root / 'scripts').mkdir()
        shutil.copyfile(ROOT / 'scripts/web-build.sh', self.root / 'scripts/web-build.sh')
        (self.root / 'web/examples').mkdir(parents=True)
        for name in ['index.html', 'favicon.svg', 'wasi_stubs.js', 'boot.js', 'platform.js', 'bridge.js', 'examples/hud.js']:
            (self.root / 'web' / name).write_text(name)
        (self.root / 'tools/wasi-sdk').mkdir(parents=True)
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        for name in ['cargo', 'wasm-bindgen', 'wasm-opt', 'brotli', 'gzip']:
            stub = self.bin / name
            stub.write_text(f'#!{sys.executable}\n' + STUB)
            stub.chmod(0o755)
        self.log = self.root / 'calls.jsonl'

    def run_build(self, profile=None, version='first', debug=False, bindgen='0.2.122'):
        env = {k: v for k, v in os.environ.items()
               if k not in ['WEB_PROFILE', 'WEB_DEBUG', 'WEB_BACKEND', 'WASI_SDK', 'WASM_OPT']}
        env.update(PATH=f'{self.bin}:/usr/bin:/bin', BUILD_TEST_LOG=str(self.log), BUILD_TEST_VERSION=version, BUILD_TEST_BINDGEN_VERSION=bindgen)
        if profile is not None:
            env['WEB_PROFILE'] = profile
        if debug:
            env['WEB_DEBUG'] = '1'
        return subprocess.run(['bash', str(self.root / 'scripts/web-build.sh')], env=env,
                              capture_output=True, text=True)

    def calls(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []

    def test_bindgen_version_mismatch_fails_before_compiling(self):
        result = self.run_build(profile='play', bindgen='0.2.121')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('does not match Cargo.lock', result.stderr)
        self.assertFalse(any(call[0] == 'cargo' for call in self.calls()))
        self.assertFalse((self.root / 'web/dist').exists())

    def test_crlf_lock_pin_is_supported_and_missing_pin_rejected(self):
        lock = self.root / 'Cargo.lock'
        lock.write_bytes(lock.read_bytes().replace(b'\n', b'\r\n'))
        result = self.run_build('play')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.log.unlink()
        lock.write_text('[[package]]\nname = "other"\nversion = "1.0"\n')
        result = self.run_build('play')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Cargo.lock (missing)', result.stderr)
        self.assertFalse(any(call[0] == 'cargo' for call in self.calls()))

    def test_default_release_and_ship_keep_shipping_postprocessing(self):
        for profile in [None, 'ship']:
            with self.subTest(profile=profile):
                self.log.unlink(missing_ok=True)
                result = self.run_build(profile)
                self.assertEqual(result.returncode, 0, result.stderr)
                calls = self.calls()
                self.assertEqual(calls[0], ['wasm-bindgen', '--version'])
                self.assertEqual(calls[1], ['cargo', 'build', '--profile', profile or 'release',
                                           '--target', 'wasm32-unknown-unknown', '-p', 'wenilla',
                                           '--no-default-features', '--features', 'webgpu'])
                self.assertIn('wasm-opt', [call[0] for call in calls])
                for suffix in ['', '.br', '.gz']:
                    self.assertEqual((self.root / f'web/dist/wenilla_bg.wasm{suffix}').read_text(), 'first-optimized')

    def test_play_reuses_profile_and_removes_stale_compressed_assets(self):
        self.assertEqual(self.run_build().returncode, 0)
        self.log.unlink()
        result = self.run_build('play', 'edited')
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.calls()
        self.assertEqual([call[0] for call in calls], ['wasm-bindgen', 'cargo', 'wasm-bindgen'])
        self.assertEqual(calls[1][3], 'play')
        dist = self.root / 'web/dist'
        self.assertEqual((dist / 'wenilla_bg.wasm').read_text(), 'edited')
        self.assertEqual((dist / 'wenilla.js').read_text(), 'edited')
        self.assertEqual(list(dist.glob('*.br')), [])
        self.assertEqual(list(dist.glob('*.gz')), [])
        self.assertTrue((dist / 'examples/hud.js').exists())
        # A later shipping build regenerates matching sidecars.
        self.assertEqual(self.run_build(version='shipping').returncode, 0)
        self.assertEqual((dist / 'wenilla_bg.wasm.br').read_text(), 'shipping-optimized')

    def test_debug_preserves_symbols_and_skips_optimizer(self):
        result = self.run_build(debug=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.calls()
        self.assertNotIn('wasm-opt', [call[0] for call in calls])
        self.assertNotIn('--remove-name-section', calls[2])
        self.assertTrue((self.root / 'web/dist/wenilla_bg.wasm.gz').exists())

    def test_invalid_profile_fails_before_build_or_output_changes(self):
        result = self.run_build('typo')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('must be play, release or ship', result.stderr)
        self.assertEqual(self.calls(), [])
        self.assertFalse((self.root / 'web/dist').exists())


if __name__ == '__main__':
    unittest.main()
