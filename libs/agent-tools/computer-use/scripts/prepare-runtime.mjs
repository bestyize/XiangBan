// Build-time only. End users run the native engine, without npm/Node.js.
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { chmod, cp, mkdir, mkdtemp, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const lock = JSON.parse(await readFile(new URL('../runtime-lock.json', import.meta.url), 'utf8'));

export function targetSpec(triple) {
  const arch = /^(aarch64|arm64)-/.test(triple) ? 'arm64' : /^x86_64-/.test(triple) ? 'x64' : null;
  const platform = /-apple-darwin$/.test(triple) ? 'darwin' : /-pc-windows-msvc$/.test(triple) ? 'win32' : /-unknown-linux-gnu$/.test(triple) ? 'linux' : null;
  if (!arch || !platform) throw new Error(`Unsupported Computer Use target: ${triple}`);
  const nativeArch = arch === 'x64' ? 'amd64' : 'arm64';
  const source = platform === 'darwin' ? 'dist/Open Computer Use.app' : `dist/${platform === 'win32' ? 'windows' : 'linux'}/${nativeArch}/open-computer-use${platform === 'win32' ? '.exe' : ''}`;
  const executable = platform === 'darwin' ? 'Open Computer Use.app/Contents/MacOS/OpenComputerUse' : path.basename(source);
  return { triple, platform, arch, key: `${platform}-${arch}`, source, executable };
}

export function selectedTarget(args, env = process.env) {
  const index = args.indexOf('--target');
  if (index >= 0 && !args[index + 1]) throw new Error('--target needs a Rust target triple');
  const host = `${process.arch === 'arm64' ? 'aarch64' : process.arch === 'x64' ? 'x86_64' : process.arch}-${process.platform === 'darwin' ? 'apple-darwin' : process.platform === 'win32' ? 'pc-windows-msvc' : 'unknown-linux-gnu'}`;
  return targetSpec(index >= 0 ? args[index + 1] : env.TAURI_ENV_TARGET_TRIPLE || host);
}

// Inspect the actual file, rather than trusting an archive directory's name.
export function binaryArchitectures(data, platform) {
  if (data.length < 64) throw new Error('Truncated native executable');
  if (platform === 'win32') {
    const offset = data.readUInt32LE(0x3c);
    if (data.toString('ascii', 0, 2) !== 'MZ' || offset + 6 > data.length || data.readUInt32LE(offset) !== 0x4550) throw new Error('Expected a Windows PE executable');
    return [({ 0x8664: 'x64', 0xaa64: 'arm64' })[data.readUInt16LE(offset + 4)]];
  }
  if (platform === 'linux') {
    if (data.toString('hex', 0, 4) !== '7f454c46' || data[4] !== 2 || data[5] !== 1) throw new Error('Expected a little-endian ELF64 executable');
    return [({ 62: 'x64', 183: 'arm64' })[data.readUInt16LE(18)]];
  }
  const cpu = n => ({ 0x01000007: 'x64', 0x0100000c: 'arm64' })[n];
  if (data.readUInt32LE(0) === 0xfeedfacf) return [cpu(data.readUInt32LE(4))];
  const magic = data.readUInt32BE(0);
  if (magic === 0xcafebabe || magic === 0xcafebabf) {
    const count = data.readUInt32BE(4), stride = magic === 0xcafebabe ? 20 : 32;
    if (count > 16 || 8 + count * stride > data.length) throw new Error('Invalid universal Mach-O executable');
    return Array.from({ length: count }, (_, i) => cpu(data.readUInt32BE(8 + i * stride)));
  }
  throw new Error('Expected a macOS Mach-O executable');
}

function run(command, args) {
  const result = spawnSync(command, args, { stdio: 'inherit', windowsHide: true });
  if (result.error || result.status !== 0) throw new Error(`${command} failed: ${result.error?.message || result.status}`);
}

async function filesIn(root) {
  const files = [];
  for (const item of await readdir(root, { withFileTypes: true })) {
    const name = path.join(root, item.name);
    if (item.isDirectory()) files.push(...await filesIn(name));
    else if (item.isFile()) files.push(name);
    else throw new Error(`Unexpected symbolic link in runtime: ${name}`);
  }
  return files;
}

export async function validateRuntime(root, spec) {
  const entries = await readdir(root);
  const allowed = new Set(['runtime.json', 'LICENSE', 'THIRD_PARTY_NOTICES.md', spec.platform === 'darwin' ? 'Open Computer Use.app' : spec.executable]);
  if (entries.some(name => !allowed.has(name))) throw new Error(`Unexpected files or another platform in Computer Use runtime: ${entries.join(', ')}`);
  const metadata = JSON.parse(await readFile(path.join(root, 'runtime.json'), 'utf8'));
  if (metadata.version !== lock.version || metadata.target !== spec.key) throw new Error(`Computer Use runtime version/target mismatch (expected ${lock.version}, ${spec.key})`);
  const arches = binaryArchitectures(await readFile(path.join(root, spec.executable)), spec.platform);
  if (arches.length !== 1 || arches[0] !== spec.arch) throw new Error(`Computer Use binary architecture mismatch: ${arches} vs ${spec.arch}`);
  if (spec.platform === 'darwin') {
    for (const file of await filesIn(path.join(root, 'Open Computer Use.app'))) {
      const data = await readFile(file);
      if (data.length < 64 || !['cffaedfe', 'cafebabe', 'cafebabf'].includes(data.toString('hex', 0, 4))) continue;
      const helpers = binaryArchitectures(data, 'darwin');
      if (helpers.length !== 1 || helpers[0] !== spec.arch) throw new Error(`Computer Use helper architecture mismatch: ${file}`);
    }
  }
  await readFile(path.join(root, 'LICENSE'));
  await readFile(path.join(root, 'THIRD_PARTY_NOTICES.md'));
}

// Replace the entire output directory so a previous platform cannot leak into a new build.
export async function stageRuntime(packageRoot, output, spec, { sign = true } = {}) {
  await mkdir(path.dirname(output), { recursive: true });
  const stage = await mkdtemp(`${output}.stage-`);
  try {
    await cp(path.join(packageRoot, spec.source), path.join(stage, spec.platform === 'darwin' ? 'Open Computer Use.app' : spec.executable), { recursive: true });
    for (const file of ['LICENSE', 'THIRD_PARTY_NOTICES.md']) await cp(path.join(packageRoot, file), path.join(stage, file));
    if (spec.platform === 'darwin') {
      if (process.platform !== 'darwin') throw new Error('Prepare macOS runtimes on macOS so architecture thinning and code signing can run');
      const app = path.join(stage, 'Open Computer Use.app');
      const nativeFiles = [];
      for (const file of await filesIn(app)) {
        const data = await readFile(file);
        // All native helpers must also match the selected architecture.
        if (data.length < 64 || !['cffaedfe', 'cafebabe', 'cafebabf'].includes(data.toString('hex', 0, 4))) continue;
        const arches = binaryArchitectures(data, 'darwin');
        if (!arches.includes(spec.arch)) throw new Error(`Wrong macOS helper architecture: ${file}`);
        if (arches.length > 1) {
          const thin = `${file}.thin`;
          run('/usr/bin/lipo', [file, '-thin', spec.arch === 'x64' ? 'x86_64' : 'arm64', '-output', thin]);
          await rename(thin, file);
        }
        await chmod(file, 0o755);
        nativeFiles.push(file);
      }
      if (sign) {
        const identity = process.env.APPLE_SIGNING_IDENTITY || '-';
        // Sign inside out with the same identity as the containing XiaoBan app.
        for (const file of nativeFiles) run('/usr/bin/codesign', ['--force', '--sign', identity, '--preserve-metadata=entitlements', file]);
        const bundles = new Set();
        for (const file of await filesIn(app)) {
          let parent = path.dirname(file);
          while (parent.startsWith(app)) {
            if (/\.(app|bundle|framework|xpc)$/.test(parent)) bundles.add(parent);
            if (parent === app) break;
            parent = path.dirname(parent);
          }
        }
        for (const bundle of [...bundles].sort((a, b) => b.length - a.length)) run('/usr/bin/codesign', ['--force', '--sign', identity, '--preserve-metadata=entitlements', bundle]);
        run('/usr/bin/codesign', ['--verify', '--deep', '--strict', app]);
      }
    } else if (spec.platform === 'linux') await chmod(path.join(stage, spec.executable), 0o755);
    await writeFile(path.join(stage, 'runtime.json'), JSON.stringify({ version: lock.version, target: spec.key, source: lock.url }, null, 2) + '\n');
    await validateRuntime(stage, spec);
    await rm(output, { recursive: true, force: true });
    await rename(stage, output);
  } finally { await rm(stage, { recursive: true, force: true }); }
}

export async function prepareRuntime(args = process.argv.slice(2)) {
  const spec = selectedTarget(args);
  const outputIndex = args.indexOf('--output');
  if (outputIndex < 0 || !args[outputIndex + 1]) throw new Error('--output <directory> is required');
  const output = path.resolve(args[outputIndex + 1]);
  if (args.includes('--check')) {
    await validateRuntime(output, spec);
    if (spec.platform === 'darwin' && process.platform === 'darwin') run('/usr/bin/codesign', ['--verify', '--deep', '--strict', path.join(output, 'Open Computer Use.app')]);
    console.log(`Verified Open Computer Use ${lock.version}: ${spec.key}`);
    return;
  }
  // A cached download is always checked again against the committed integrity.
  const cache = path.join(tmpdir(), 'xiaoban-computer-use', lock.version);
  const archive = path.join(cache, 'package.tgz');
  await mkdir(cache, { recursive: true });
  let data;
  try { data = await readFile(archive); } catch {}
  const valid = value => value && `sha512-${createHash('sha512').update(value).digest('base64')}` === lock.integrity;
  if (!valid(data)) {
    console.log(`Downloading Open Computer Use ${lock.version} for ${spec.key} (build time only)…`);
    const response = await fetch(lock.url, { signal: AbortSignal.timeout(180_000) });
    if (!response.ok) throw new Error(`Runtime download failed: HTTP ${response.status}`);
    data = Buffer.from(await response.arrayBuffer());
    if (!valid(data)) throw new Error('Open Computer Use archive integrity mismatch');
    await writeFile(archive, data);
  }
  const unpack = await mkdtemp(path.join(cache, 'extract-'));
  try {
    // Extract only the selected native artifact plus attribution. Never run npm lifecycle scripts.
    run('tar', ['-xzf', archive, '-C', unpack, `package/${spec.source}`, 'package/LICENSE', 'package/THIRD_PARTY_NOTICES.md']);
    await stageRuntime(path.join(unpack, 'package'), output, spec);
    console.log(`Bundled Open Computer Use ${lock.version}: ${spec.key}`);
  } finally { await rm(unpack, { recursive: true, force: true }); }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  prepareRuntime().catch(error => { console.error(error.message); process.exitCode = 1; });
}
