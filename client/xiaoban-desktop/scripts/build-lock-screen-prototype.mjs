import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { signMacos } from '../../../.github/signing/identity.mjs';

const root = fileURLToPath(new URL('..', import.meta.url));
const source = join(root, 'src-tauri/macos/lock-screen-prototype');
const [inputArg, outputArg] = process.argv.slice(2);

function execute(executable, arguments_, options = {}) {
  return execFileSync(executable, arguments_, { stdio: 'inherit', ...options });
}

try {
  if (!inputArg || !outputArg) {
    throw new Error('用法：node scripts/build-lock-screen-prototype.mjs 小伴.app 小伴锁屏原型.dmg');
  }
  if (process.platform !== 'darwin' || process.arch !== 'arm64') {
    throw new Error('锁屏原型需要装有 Xcode 的 Apple 芯片 Mac。');
  }
  const input = resolve(inputArg);
  const output = resolve(outputArg);
  if (!input.endsWith('.app') || !output.endsWith('.dmg') || existsSync(output)) {
    throw new Error('请指定小伴 .app 和一个尚不存在的 .dmg 输出路径。');
  }
  const info = join(input, 'Contents/Info.plist');
  const identifier = execute('/usr/bin/plutil', ['-extract', 'CFBundleIdentifier', 'raw', info],
    { encoding: 'utf8', stdio: 'pipe' }).trim();
  if (identifier !== 'xyz.xiaban.desktop-wallpaper') throw new Error('输入必须是小伴桌面安装包里的应用。');
  execute('/usr/bin/codesign', ['--verify', '--strict', input]);

  const staging = mkdtempSync(join(dirname(output), '.xiaoban-lock-screen-'));
  try {
    const host = join(staging, '小伴.app');
    execute('/usr/bin/ditto', [input, host]);
    const extension = join(host, 'Contents/PlugIns/XiaobanLockScreenPrototype.appex');
    const binaries = join(extension, 'Contents/MacOS');
    const resources = join(extension, 'Contents/Resources');
    mkdirSync(binaries, { recursive: true });
    mkdirSync(resources, { recursive: true });
    copyFileSync(join(source, 'Info.plist'), join(extension, 'Contents/Info.plist'));
    copyFileSync(join(root, 'public/wallpapers/builtin/sea-breeze.mp4'), join(resources, 'scene.mp4'));
    copyFileSync(join(root, 'src-tauri/icons/icon.png'), join(resources, 'preview.png'));
    copyFileSync(join(source, 'LICENSE-Phosphene.txt'), join(resources, 'LICENSE-Phosphene.txt'));

    execute('/usr/bin/xcrun', ['--sdk', 'macosx', 'swiftc', '-target', 'arm64-apple-macosx26.0',
      '-swift-version', '5', '-parse-as-library', '-application-extension', '-O',
      '-import-objc-header', join(source, 'WallpaperBridge.h'),
      join(source, 'SettingsSchema.swift'), join(source, 'Prototype.swift'),
      '-o', join(binaries, 'XiaobanLockScreenPrototype')]);
    signMacos(extension, ['--options', 'runtime', '--entitlements', join(source, 'Entitlements.plist')]);
    signMacos(host, ['--preserve-metadata=identifier,entitlements,requirements,flags']);
    execute('/usr/bin/codesign', ['--verify', '--deep', '--strict', host]);
    execute('/usr/bin/hdiutil', ['create', '-volname', '小伴锁屏原型', '-srcfolder', staging, '-format', 'UDZO', output]);
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
}
