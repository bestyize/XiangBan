import assert from 'node:assert/strict';
import test from 'node:test';
import {
  BRAND_ICON_URL,
  DEFAULT_ASSET_BASE,
  DESKTOP_URL,
  FILTERS,
  MOBILE_URL,
  PRODUCTS,
  WALLPAPER_URL,
} from '../src/data/products.ts';

test('catalog contains all 6 expected products in order', () => {
  assert.deepEqual(
    PRODUCTS.map((p) => p.id),
    ['mobile', 'desktop', 'community', 'game', 'people', 'wallpaper']
  );
  assert.equal(FILTERS.find((f) => f.key === 'all')?.count, PRODUCTS.length);
});

test('desktop and mobile landing pages are hosted under xiaban.xyz and wallpaper is standalone', () => {
  assert.equal(MOBILE_URL, 'https://xiaban.xyz/app');
  assert.equal(DESKTOP_URL, 'https://xiaban.xyz/desktop');
  assert.equal(WALLPAPER_URL, 'https://wallpaper.xiangban.xyz/');
});

test('all images are WebP hosted on res.xiangban.xyz', () => {
  const urls: string[] = [BRAND_ICON_URL];
  for (const p of PRODUCTS) {
    urls.push(p.previewImages.primary);
    if (p.previewImages.secondary) urls.push(p.previewImages.secondary);
    if (p.previewImages.tertiary) urls.push(p.previewImages.tertiary);
    for (const s of p.shots) urls.push(s.src);
  }
  for (const url of urls) {
    assert.ok(url.startsWith(`${DEFAULT_ASSET_BASE}/`), `Expected resource host URL: ${url}`);
    assert.ok(url.endsWith('.webp'), `Expected WebP image: ${url}`);
  }
});

test('all product links use HTTPS', () => {
  for (const p of PRODUCTS) {
    assert.ok(p.url.startsWith('https://'), `Expected HTTPS product URL: ${p.url}`);
  }
});
