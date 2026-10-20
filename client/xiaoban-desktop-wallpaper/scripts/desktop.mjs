import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { ensureSigningIdentity, signMacos, verifyStableSignature } from '../../../.github/signing/identity.mjs';

const root = fileURLToPath(new URL('..', import.meta.url));
try {
  const env = { ...process.env };
  if (process.platform === 'darwin') {
    env.APPLE_SIGNING_IDENTITY = ensureSigningIdentity();
    env.XIAOBAN_SIGNING_KEYCHAIN = process.env.XIAOBAN_SIGNING_KEYCHAIN;
    // cargo run invokes this after every rebuild, before executing the app.
    if (process.argv[2] === 'dev') {
      // Cargo splits runner strings on whitespace. Use PATH lookup so project paths with spaces work.
      env.PATH = fileURLToPath(new URL('.', import.meta.url)) + path.delimiter + env.PATH;
      env.CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER = 'macos-dev-runner.mjs';
      env.CARGO_TARGET_X86_64_APPLE_DARWIN_RUNNER = 'macos-dev-runner.mjs';
    }
  }
  const child = spawn(process.execPath, [path.join(root, 'node_modules/@tauri-apps/cli/tauri.js'), ...process.argv.slice(2)], { cwd: root, env, stdio: 'inherit' });
  for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => child.kill(signal));
  child.on('error', error => { console.error(error.message); process.exitCode = 1; });
  child.on('exit', code => {
    try {
      if (code === 0 && process.platform === 'darwin' && process.argv[2] === 'build') {
        const args = process.argv.slice(2);
        const target = args.includes('--target') ? args[args.indexOf('--target') + 1] : '';
        const profile = args.includes('--debug') ? 'debug' : args.includes('--profile') ? args[args.indexOf('--profile') + 1] : 'release';
        const targetDir = process.env.CARGO_TARGET_DIR ? path.resolve(root, 'src-tauri', process.env.CARGO_TARGET_DIR) : path.join(root, 'src-tauri/target');
        const output = path.join(targetDir, target, profile);
        if (process.argv.includes('--no-bundle')) signMacos(path.join(output, 'xiaoban-desktop-wallpaper'), ['--identifier', 'xyz.xiaban.desktop-wallpaper']);
        else {
          const app = path.join(output, 'bundle/macos/小伴.app');
          verifyStableSignature(app);
          verifyStableSignature(path.join(app, 'Contents/Resources/agent-tools/computer-use/Open Computer Use.app'));
        }
      }
      process.exitCode = code ?? 1;
    } catch (error) { console.error(error.message); process.exitCode = 1; }
  });
} catch (error) { console.error(error.message); process.exitCode = 1; }
