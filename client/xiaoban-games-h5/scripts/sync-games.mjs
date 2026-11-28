#!/usr/bin/env node
import { cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const appRoot = resolve(__dirname, '..');
const repoRoot = resolve(appRoot, '../..');
const sourcePacksDir = join(repoRoot, 'client/xiaoban-desktop/public/game-packs');
const plazaIconsDir = join(repoRoot, 'client/xiaoban-multiplatform/docs/design-specs/prototypes/app-plaza-assets/plaza');
const outPacksDir = join(appRoot, 'public/game-packs');
const outIconsDir = join(appRoot, 'public/game-icons');

const GAME_IDS = [
  'ink-blade',
  'water-sort',
  'dusk-glider',
  'block-puzzle',
  'merge-2048',
  'snake-arena',
  'pro-cube',
  'jade-gomoku'
];

const ICON_FILES = [
  'ink-blade.png', 'ink-blade-cover.svg',
  'water-sort.webp', 'water-sort-cover.svg',
  'dusk-glider.png', 'dusk-glider-cover.svg',
  'block-puzzle.webp', 'block-puzzle-cover.svg',
  'merge-2048.png', 'merge-2048-cover.svg',
  'snake-arena.png', 'snake-arena-cover.svg',
  'pro-cube.png', 'pro-cube-cover.svg',
  'jade-gomoku.png', 'jade-gomoku-cover.svg'
];

const BRIDGE_TAG = '<script src="../../xb-bridge.js"></script>';

function injectBridge(html, gameId) {
  const tag = `<script>window.__XB_GAME_ID__=${JSON.stringify(gameId)};</script>${BRIDGE_TAG}`;
  if (html.includes('xb-bridge.js')) return html;
  if (/<head[^>]*>/i.test(html)) {
    return html.replace(/<head([^>]*)>/i, `<head$1>\n${tag}`);
  }
  return `${tag}\n${html}`;
}

async function main() {
  if (!existsSync(sourcePacksDir)) {
    throw new Error(`找不到小伴游戏包源目录: ${sourcePacksDir}`);
  }

  await rm(outPacksDir, { recursive: true, force: true });
  await rm(outIconsDir, { recursive: true, force: true });
  await mkdir(outPacksDir, { recursive: true });
  await mkdir(outIconsDir, { recursive: true });

  for (const id of GAME_IDS) {
    const src = join(sourcePacksDir, id);
    const dst = join(outPacksDir, id);
    if (!existsSync(src)) {
      throw new Error(`缺少官方游戏包: ${src}`);
    }
    await cp(src, dst, { recursive: true });

    const entryPath = join(dst, 'index.html');
    if (existsSync(entryPath)) {
      const raw = await readFile(entryPath, 'utf8');
      await writeFile(entryPath, injectBridge(raw, id), 'utf8');
    }
  }

  for (const file of ICON_FILES) {
    const src = join(plazaIconsDir, file);
    if (existsSync(src)) {
      await cp(src, join(outIconsDir, file));
    }
  }

  console.log(`[sync-games] 已同步 ${GAME_IDS.length} 款小伴官方游戏包与高清图标至 public/`);
}

main().catch((err) => {
  console.error('[sync-games] 同步失败:', err);
  process.exit(1);
});
