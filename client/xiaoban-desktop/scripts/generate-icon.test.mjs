import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import test from 'node:test';

const svg = await readFile(fileURLToPath(new URL('../src-tauri/icons/icon-source.svg', import.meta.url)), 'utf8');
const darkSvg = await readFile(fileURLToPath(new URL('../src-tauri/icons/icon-source-dark.svg', import.meta.url)), 'utf8');
const config = JSON.parse(await readFile(fileURLToPath(new URL('../src-tauri/tauri.conf.json', import.meta.url)), 'utf8'));

test('desktop app icon uses the Twin Stars mark in pure white over Sunset Coral glaze and Obsidian dark counterpart', () => {
  assert.match(svg, /stop-color="#FF6584"/);
  assert.match(svg, /stop-color="#F9744B"/);
  assert.match(svg, /fill="#FFFFFF"/);
  assert.match(svg, /translate\(55\.5 62\.5\) scale\(0\.96\)/);
  assert.match(svg, /translate\(82\.5 35\.5\) scale\(0\.47\)/);

  assert.match(darkSvg, /stop-color="#262023"/);
  assert.match(darkSvg, /stop-color="#121012"/);
  assert.match(darkSvg, /fill="#FFFFFF"/);
  assert.match(darkSvg, /translate\(55\.5 62\.5\) scale\(0\.96\)/);
  assert.match(darkSvg, /translate\(82\.5 35\.5\) scale\(0\.47\)/);
});

test('Tauri runs icon generation before building and includes Windows/Linux images', () => {
  assert.match(config.build.beforeBuildCommand, /generate-icon\.mjs/);
  assert.ok(config.bundle.icon.includes('icons/icon.ico'));
  assert.ok(config.bundle.icon.includes('icons/icon.png'));
});

test('checked-in native icons are generated from the current Sunset Coral source, not left from an older design', async () => {
  const tray = await readFile(fileURLToPath(new URL('../src-tauri/icons/tray.rgba', import.meta.url)));
  assert.equal(tray.length, 64 * 64 * 4);
  // Left-middle background pixel sits inside the Sunset Coral gradient (high red, mid green/blue, opaque).
  const at = (32 * 64 + 6) * 4;
  const [r, g, b, a] = [tray[at], tray[at + 1], tray[at + 2], tray[at + 3]];
  assert.equal(a, 255, 'Expected opaque background pixel in tray.rgba');
  assert.ok(r >= 245 && g >= 100 && g <= 140 && b >= 100 && b <= 145,
    `Expected Sunset Coral pixel at left-middle, got rgb(${r},${g},${b}); run \`node scripts/generate-icon.mjs\``);
  // Center of the main star (~(55.5/120*64, 62.5/120*64) ≈ (30, 33)) must be crisp pure white.
  const centerAt = (33 * 64 + 30) * 4;
  assert.deepEqual(Array.from(tray.subarray(centerAt, centerAt + 4)), [255, 255, 255, 255],
    'Expected pure white main star center in tray.rgba');
});
