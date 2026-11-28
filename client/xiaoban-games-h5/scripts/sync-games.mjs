#!/usr/bin/env node
import { execFileSync } from 'node:child_process';
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

const ICON_SOURCES = {
  'ink-blade': 'ink-blade.png',
  'water-sort': 'water-sort.png',
  'dusk-glider': 'dusk-glider.png',
  'block-puzzle': 'block-puzzle.webp',
  'merge-2048': 'merge-2048.png',
  'snake-arena': 'snake-arena.png',
  'pro-cube': 'pro-cube.png',
  'jade-gomoku': 'jade-gomoku.png'
};

function hasCwebp() {
  try {
    execFileSync('cwebp', ['-version'], { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

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

  const canEncodeWebp = hasCwebp();

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

    // 同步 16:9 矢量封面
    const coverSrc = join(plazaIconsDir, `${id}-cover.svg`);
    if (existsSync(coverSrc)) {
      await cp(coverSrc, join(outIconsDir, `${id}-cover.svg`));
    }

    // 同步并压缩 1:1 图标为 160x160 高清 WebP
    const iconSrc = join(plazaIconsDir, ICON_SOURCES[id]);
    const iconDst = join(outIconsDir, `${id}.webp`);
    if (existsSync(iconSrc)) {
      if (canEncodeWebp) {
        execFileSync('cwebp', ['-quiet', '-q', '82', '-m', '6', '-resize', '160', '160', iconSrc, '-o', iconDst]);
      } else {
        await cp(iconSrc, iconDst);
      }
    }
  }

  console.log(`[sync-games] 已同步 ${GAME_IDS.length} 款官方游戏包与${canEncodeWebp ? '压缩 WebP ' : ''}图标至 public/`);
}

main().catch((err) => {
  console.error('[sync-games] 同步失败:', err);
  process.exit(1);
});
