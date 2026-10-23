import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import test from 'node:test';

const svg = await readFile(fileURLToPath(new URL('../src-tauri/icons/icon-source.svg', import.meta.url)), 'utf8');
const appearance = await readFile(fileURLToPath(new URL('../src/ui/appearance.ts', import.meta.url)), 'utf8');
const config = JSON.parse(await readFile(fileURLToPath(new URL('../src-tauri/tauri.conf.json', import.meta.url)), 'utf8'));

test('desktop app icon reuses the 小伴 AI Sparkles mark and current theme red', () => {
  const match = appearance.match(/id: 'red', name: '小伴红',[\s\S]*?light: \{ btn: '(#[0-9A-Fa-f]{6})'/);
  assert.ok(match, 'Cannot find the default red theme palette');
  assert.match(appearance, /DEFAULT_ACCENT: AccentId = 'red'/);
  assert.ok(svg.includes(`fill="${match[1]}"`), 'App icon red differs from the default theme');
  assert.match(svg, /stroke="#FFFFFF"/);
  assert.match(svg, /M11\.017 2\.814a1 1 0 0 1 1\.966 0/);
  assert.match(svg, /<path d="M20 2v4"\s*\/>/);
  assert.match(svg, /<circle cx="4" cy="20" r="2"\s*\/>/);
});

test('Tauri runs icon generation before building and includes Windows/Linux images', () => {
  assert.match(config.build.beforeBuildCommand, /generate-icon\.mjs/);
  assert.match(config.build.beforeDevCommand, /generate-icon\.mjs/);
  assert.ok(config.bundle.icon.includes('icons/icon.ico'));
  assert.ok(config.bundle.icon.includes('icons/icon.png'));
});

test('checked-in native icons are generated from the current source, not left from an older design', async () => {
  const red = appearance.match(/id: 'red', name: '小伴红',[\s\S]*?light: \{ btn: '#([0-9A-Fa-f]{6})'/)[1];
  const tray = await readFile(fileURLToPath(new URL('../src-tauri/icons/tray.rgba', import.meta.url)));
  assert.equal(tray.length, 64 * 64 * 4);
  // The Sparkles mark leaves the left-middle of the rounded square plain theme red.
  const at = (32 * 64 + 6) * 4;
  assert.equal(tray.subarray(at, at + 3).toString('hex').toUpperCase(), red.toUpperCase(),
    'Run `node scripts/generate-icon.mjs` and commit icon.icns, icon.ico, icon.png and tray.rgba');
});
