import assert from 'node:assert/strict';
import { readFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const base = 'client/xiaoban-multiplatform/androidApp/src/main/';
const read = path => readFileSync(resolve(root, path), 'utf8');
const html = read(base + 'assets/game-packs-v2/quick-launch/index.html');
const native = read(base + 'kotlin/xyz/xiangban/xiaoban/QuickLaunchShortcut.kt');
const manifest = read(base + 'AndroidManifest.xml');
const appsSource = html.match(/const apps=\[([\s\S]*?)\];/)?.[1];
assert.ok(appsSource, 'missing Quick Launch catalog');
const webNames = [...appsSource.matchAll(/(?:^|[,{])\s*(?:name|"name")\s*:\s*(?:"([^"]+)"|'([^']+)')/g)]
  .map(match => match[1] ?? match[2]);
const nativeEntries = [...native.matchAll(/Target\("([^"]+)", "([^"]+\.webp)"/g)];
const nativeNames = nativeEntries.map(match => match[1]);
assert.equal(webNames.length, 104, 'web catalog should list 104 apps');
assert.equal(new Set(webNames).size, 104, 'web catalog contains duplicate names');
assert.equal(nativeNames.length, 104, 'native catalog should list 104 apps');
assert.deepEqual(nativeNames, webNames, 'native and web catalog ordering/names differ');
assert.ok(!/(?:scheme|"scheme")\s*:/.test(appsSource), 'home launch must not depend on URL schemes');
for (const [, name, image] of nativeEntries) {
  assert.ok(existsSync(resolve(root, base + 'assets/game-packs-v2/quick-launch/icons/' + image)),
    'missing icon for ' + name + ': ' + image);
}
assert.match(native, /getLaunchIntentForPackage/, 'known app packages must use Android launcher');
assert.match(native, /bilibili:\/\/home/, 'Bilibili must use its home deep link with launcher fallback');
assert.match(native, /queryIntentActivities/, 'unknown packages must resolve installed launchers');
assert.match(native, /appleOnly/, 'unsupported Apple-only entries must not start arbitrary apps');
assert.match(manifest, /android\.intent\.category\.LAUNCHER/, 'launcher package visibility is required');
assert.ok(!/<uses-permission[^>]*QUERY_ALL_PACKAGES/.test(manifest), 'do not request broad package inventory access');
assert.match(html, /quickLaunch\.available/, 'detail must check installed launchability');
console.log('Quick Launch: 104 matching apps, launcher resolution, icons and manifest verified.');
