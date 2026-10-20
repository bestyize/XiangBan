// Static DMG layout contract. Real Finder icon geometry still needs macOS visual QA.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const read = (relative) => readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf8');
const conf = JSON.parse(read('../src-tauri/tauri.macos.conf.json'));
const render = read('../src-tauri/dmg/render-background.swift');
const dmg = conf.bundle.macOS.dmg;

assert.equal(dmg.background, 'dmg/background.png');
assert.deepEqual(dmg.windowSize, { width: 760, height: 440 });
assert.deepEqual(dmg.appPosition, { x: 177, y: 220 });
assert.deepEqual(dmg.applicationFolderPosition, { x: 583, y: 220 });
assert.match(conf.build.beforeBundleCommand, /^swift src-tauri\/dmg\/render-background\.swift src-tauri\/dmg\/background\.png$/);
assert.equal(conf.bundle.macOS.signingIdentity, '-'); // CI sets APPLE_SIGNING_IDENTITY; unchanged
assert.equal(conf.bundle.macOS.minimumSystemVersion, '12.0');
assert.match(render, /private let width = 760/);
assert.match(render, /private let height = 440/);
assert.match(render, /把小伴带回桌面/);
assert.match(render, /拖动以安装/);
assert.match(render, /仍要打开/);
assert.doesNotMatch(render, /sudo |spctl |xattr -d |tccutil reset/);
console.log('macOS DMG Chinese instruction layout contract OK');
