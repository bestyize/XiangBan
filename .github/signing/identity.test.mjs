import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { SIGNING_NAME, signingIdentity, signMacos } from './identity.mjs';

test('local builds default to the persistent certificate and reject ad-hoc overrides', () => {
  assert.equal(signingIdentity({}), SIGNING_NAME);
  assert.equal(signingIdentity({ APPLE_SIGNING_IDENTITY: '  ' }), SIGNING_NAME);
  assert.throws(() => signingIdentity({ APPLE_SIGNING_IDENTITY: '-' }), /Ad-hoc/);
  assert.equal(signingIdentity({ APPLE_SIGNING_IDENTITY: 'explicit-developer-id' }), 'explicit-developer-id');
});

test('changing native code keeps the same designated requirement', {
  skip: process.platform !== 'darwin' || process.env.XIAOBAN_TEST_SIGNING !== '1',
}, t => {
  const root = mkdtempSync(path.join(tmpdir(), 'xiaoban-signing-test-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const app = path.join(root, 'fixture.app');
  const contents = path.join(app, 'Contents');
  mkdirSync(path.join(contents, 'MacOS'), { recursive: true });
  writeFileSync(path.join(contents, 'Info.plist'), '<?xml version="1.0"?><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>xyz.xiaban.signing-fixture</string><key>CFBundleExecutable</key><string>fixture</string><key>CFBundlePackageType</key><string>APPL</string></dict></plist>');
  const requirement = () => execFileSync('/usr/bin/codesign', ['--display', '--requirements', '-', app], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  copyFileSync('/usr/bin/true', path.join(contents, 'MacOS/fixture'));
  signMacos(app);
  const before = requirement();
  copyFileSync('/usr/bin/false', path.join(contents, 'MacOS/fixture'));
  signMacos(app);
  assert.equal(requirement(), before);
  assert.doesNotMatch(before, /cdhash/);
  assert.match(before, /certificate root/);
});

test('Cargo development launch runs a signed app bundle, including paths with spaces', {
  skip: process.platform !== 'darwin' || process.env.XIAOBAN_TEST_SIGNING !== '1',
}, t => {
  const root = mkdtempSync(path.join(tmpdir(), 'xiaoban dev launch '));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(path.join(root, 'src'));
  writeFileSync(path.join(root, 'Cargo.toml'), '[package]\nname="signed-launch-fixture"\nversion="0.1.0"\nedition="2021"\n');
  writeFileSync(path.join(root, 'src/main.rs'), 'fn main() { println!("signed dev launch works"); }');
  const scripts = fileURLToPath(new URL('../../client/xiaoban-desktop/scripts/', import.meta.url));
  const result = execFileSync('cargo', ['run', '--offline', '--quiet', '--manifest-path', path.join(root, 'Cargo.toml')], {
    encoding: 'utf8', timeout: 30_000, stdio: ['ignore', 'pipe', 'pipe'],
    env: { ...process.env, CARGO_TARGET_DIR: path.join(root, 'target'), PATH: scripts + path.delimiter + process.env.PATH,
      CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER: 'macos-dev-runner.mjs', CARGO_TARGET_X86_64_APPLE_DARWIN_RUNNER: 'macos-dev-runner.mjs' },
  });
  assert.match(result, /signed dev launch works/);
  const app = path.join(root, 'target/debug/xiaoban-dev/小伴.app');
  const requirement = execFileSync('/usr/bin/codesign', ['--display', '--requirements', '-', app], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  assert.match(requirement, /identifier "xyz.xiaban.desktop-wallpaper"/);
  assert.doesNotMatch(requirement, /cdhash/);
});
