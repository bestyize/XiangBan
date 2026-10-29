#!/usr/bin/env node
// Cargo target runner: launch a real app bundle with a stable identity after each rebuild.
import { spawn, spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { signMacos } from '../../../.github/signing/identity.mjs';

export function stageDevApp(binary) {
  const tauri = fileURLToPath(new URL('../src-tauri/', import.meta.url));
  const app = path.join(path.dirname(binary), 'xiaoban-dev/小伴.app');
  const contents = path.join(app, 'Contents');
  rmSync(app, { recursive: true, force: true });
  mkdirSync(path.join(contents, 'MacOS'), { recursive: true });
  mkdirSync(path.join(contents, 'Resources'), { recursive: true });
  const executable = path.join(contents, 'MacOS/xiaoban-desktop');
  copyFileSync(binary, executable);
  // icon.icns is generated, not tracked; `tauri dev` does not run the generator, so make it on first use.
  const icns = path.join(tauri, 'icons/icon.icns');
  if (!existsSync(icns)) {
    const made = spawnSync(process.execPath, [fileURLToPath(new URL('./generate-icon.mjs', import.meta.url))], { stdio: 'inherit' });
    if (made.status !== 0) throw new Error('icon.icns is missing and scripts/generate-icon.mjs failed');
  }
  copyFileSync(icns, path.join(contents, 'Resources/icon.icns'));
  const config = JSON.parse(readFileSync(path.join(tauri, 'tauri.conf.json'), 'utf8'));
  const extra = readFileSync(path.join(tauri, 'Info.plist'), 'utf8');
  // Keep privacy purpose strings from the release bundle. Development data remains isolated in Rust.
  const keys = `<key>CFBundleIdentifier</key><string>${config.identifier}</string>
<key>CFBundleName</key><string>小伴</string>
<key>CFBundleExecutable</key><string>xiaoban-desktop</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>${config.version}</string>
<key>CFBundleVersion</key><string>${config.version}</string>
<key>CFBundleIconFile</key><string>icon.icns</string>`;
  writeFileSync(path.join(contents, 'Info.plist'), extra.replace('</dict>', `${keys}\n</dict>`));
  signMacos(app, ['--entitlements', path.join(tauri, 'Entitlements.plist')]);
  return executable;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const executable = stageDevApp(path.resolve(process.argv[2]));
    const env = Object.fromEntries(Object.entries(process.env).filter(([, value]) => value !== undefined));
    if (process.execve) process.execve(executable, [executable, ...process.argv.slice(3)], env);
    else {
      const child = spawn(executable, process.argv.slice(3), { stdio: 'inherit', env });
      for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => child.kill(signal));
      child.on('error', error => { console.error(error.message); process.exitCode = 1; });
      child.on('exit', code => { process.exitCode = code ?? 1; });
    }
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
