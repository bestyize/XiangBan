import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { binaryArchitectures, lock, selectedTarget, stageRuntime, targetSpec, validateRuntime } from './prepare-runtime.mjs';

function pe(machine) {
  const data = Buffer.alloc(128);
  data.write('MZ'); data.writeUInt32LE(64, 0x3c);
  data.writeUInt32LE(0x4550, 64); data.writeUInt16LE(machine, 68);
  return data;
}

test('uses the compilation target, including cross builds; rejects Android and unknown architectures', () => {
  assert.equal(selectedTarget([], { TAURI_ENV_TARGET_TRIPLE: 'aarch64-apple-darwin' }).key, 'darwin-arm64');
  assert.equal(selectedTarget(['--target', 'x86_64-pc-windows-msvc'], { TAURI_ENV_TARGET_TRIPLE: 'aarch64-apple-darwin' }).key, 'win32-x64');
  assert.equal(targetSpec('aarch64-pc-windows-msvc').source, 'dist/windows/arm64/open-computer-use.exe');
  assert.equal(targetSpec('aarch64-unknown-linux-gnu').source, 'dist/linux/arm64/open-computer-use');
  assert.throws(() => targetSpec('aarch64-linux-android'), /Unsupported/);
  assert.throws(() => targetSpec('i686-pc-windows-msvc'), /Unsupported/);
  assert.throws(() => selectedTarget(['--target'], {}), /needs/);
});

test('checks native formats and architectures, rather than file extensions', () => {
  assert.deepEqual(binaryArchitectures(pe(0x8664), 'win32'), ['x64']);
  assert.deepEqual(binaryArchitectures(pe(0xaa64), 'win32'), ['arm64']);
  assert.throws(() => binaryArchitectures(pe(0x8664), 'darwin'), /Mach-O/);
  assert.throws(() => binaryArchitectures(pe(0x8664), 'linux'), /ELF/);
  const macho = Buffer.alloc(64);
  macho.writeUInt32LE(0xfeedfacf); macho.writeUInt32LE(0x0100000c, 4);
  assert.deepEqual(binaryArchitectures(macho, 'darwin'), ['arm64']);
  const fat = Buffer.alloc(64);
  fat.writeUInt32BE(0xcafebabe); fat.writeUInt32BE(2, 4);
  fat.writeUInt32BE(0x01000007, 8); fat.writeUInt32BE(0x0100000c, 28);
  assert.deepEqual(binaryArchitectures(fat, 'darwin'), ['x64', 'arm64']);
});

async function fixture(t, machine = 0x8664) {
  const root = await mkdtemp(path.join(tmpdir(), 'ocu-stage-test-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const pkg = path.join(root, 'package'), output = path.join(root, 'runtime');
  await mkdir(path.join(pkg, 'dist/windows/amd64'), { recursive: true });
  await writeFile(path.join(pkg, 'dist/windows/amd64/open-computer-use.exe'), pe(machine));
  await mkdir(path.join(pkg, 'dist/linux/amd64'), { recursive: true });
  await writeFile(path.join(pkg, 'dist/linux/amd64/open-computer-use'), 'must not be copied');
  await writeFile(path.join(pkg, 'LICENSE'), 'MIT fixture');
  await writeFile(path.join(pkg, 'THIRD_PARTY_NOTICES.md'), 'Notices fixture');
  return { pkg, output };
}

test('stages only the target payload and removes a previous platform completely', async t => {
  const { pkg, output } = await fixture(t);
  await mkdir(path.join(output, 'Open Computer Use.app'), { recursive: true });
  await writeFile(path.join(output, 'old-linux-binary'), 'stale');
  const spec = targetSpec('x86_64-pc-windows-msvc');
  await stageRuntime(pkg, output, spec, { sign: false });
  assert.deepEqual((await readdir(output)).sort(), ['LICENSE', 'THIRD_PARTY_NOTICES.md', 'open-computer-use.exe', 'runtime.json']);
  assert.deepEqual(JSON.parse(await readFile(path.join(output, 'runtime.json'), 'utf8')), { version: lock.version, target: 'win32-x64', source: lock.url });
  await validateRuntime(output, spec);
});

test('a mislabeled CPU binary blocks staging and preserves the previous runtime', async t => {
  const { pkg, output } = await fixture(t, 0xaa64);
  await mkdir(output);
  await writeFile(path.join(output, 'previous'), 'keep');
  await assert.rejects(stageRuntime(pkg, output, targetSpec('x86_64-pc-windows-msvc')), /architecture mismatch/);
  assert.equal(await readFile(path.join(output, 'previous'), 'utf8'), 'keep');
});

test('pre-bundle verification rejects mixed payloads and stale metadata', async t => {
  const { pkg, output } = await fixture(t);
  const spec = targetSpec('x86_64-pc-windows-msvc');
  await stageRuntime(pkg, output, spec);
  await writeFile(path.join(output, 'open-computer-use'), 'foreign');
  await assert.rejects(validateRuntime(output, spec), /another platform/);
  await rm(path.join(output, 'open-computer-use'));
  await writeFile(path.join(output, 'runtime.json'), JSON.stringify({ version: lock.version, target: 'win32-arm64' }));
  await assert.rejects(validateRuntime(output, spec), /version\/target mismatch/);
});

test('macOS verification checks nested helper architecture as well as the main app', async t => {
  const root = await mkdtemp(path.join(tmpdir(), 'ocu-mac-test-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const app = path.join(root, 'Open Computer Use.app/Contents');
  await mkdir(path.join(app, 'MacOS'), { recursive: true });
  await mkdir(path.join(app, 'Resources'), { recursive: true });
  const native = cpu => { const b = Buffer.alloc(64); b.writeUInt32LE(0xfeedfacf); b.writeUInt32LE(cpu, 4); return b; };
  await writeFile(path.join(app, 'MacOS/OpenComputerUse'), native(0x0100000c));
  await writeFile(path.join(app, 'Resources/Helper'), native(0x01000007));
  await writeFile(path.join(root, 'runtime.json'), JSON.stringify({ version: lock.version, target: 'darwin-arm64' }));
  await writeFile(path.join(root, 'LICENSE'), 'MIT');
  await writeFile(path.join(root, 'THIRD_PARTY_NOTICES.md'), 'Notices');
  await assert.rejects(validateRuntime(root, targetSpec('aarch64-apple-darwin')), /helper architecture mismatch/);
  await writeFile(path.join(app, 'Resources/Helper'), native(0x0100000c));
  await validateRuntime(root, targetSpec('aarch64-apple-darwin'));
});
