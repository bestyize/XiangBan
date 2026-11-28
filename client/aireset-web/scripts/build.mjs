#!/usr/bin/env node
/**
 * AI额度监控（QuotaReset）静态站构建脚本，零依赖。
 *
 * 1. 从 src/index.html 生成两个入口页：`/`（重置雷达）与 `/benchmark/`（订阅额度实测），
 *    各自带独立 title / description / canonical / og:url，便于搜索引擎分别收录。
 * 2. 用本机 Chromium 内核浏览器（Edge / Chrome / Chromium）无头预渲染，把脚本生成的表格与卡片
 *    直接写进静态 HTML，爬虫无需执行 JS 也能读到全部内容；页面脚本仍保留，浏览器端会接管交互。
 * 3. 生成 favicon.svg / robots.txt / sitemap.xml / 404.html。
 *
 * 用法：node scripts/build.mjs [--no-prerender]
 * 可选变量：BROWSER_BIN 指定浏览器可执行文件；SITE_ORIGIN 覆盖站点域名。
 */
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SRC = path.join(ROOT, 'src', 'index.html');
const DIST = path.join(ROOT, 'dist');
const ORIGIN = (process.env.SITE_ORIGIN || 'https://aireset.xiangban.xyz').replace(/\/$/, '');
const NO_PRERENDER = process.argv.includes('--no-prerender');

const PAGES = [
  {
    out: 'index.html',
    urlPath: '/',
    tab: 'overview',
    title: 'AI额度监控 QuotaReset — ChatGPT Codex、反重力 Antigravity、Claude 额度重置雷达',
    description:
      'AI额度监控（QuotaReset）实时追踪 ChatGPT/Codex、反重力 Antigravity 与 Claude 的官方额度全局重置与重置卡动态，并对比各档 AI 订阅的 Token 容量与官方 API 等值。',
    ogTitle: 'AI额度监控 QuotaReset — ChatGPT Codex、反重力、Claude 额度重置雷达',
    ogDescription:
      '一站追踪 ChatGPT/Codex、反重力 Antigravity、Claude 额度重置动态与主流 AI 订阅套餐实测性价比。',
  },
  {
    out: path.join('benchmark', 'index.html'),
    urlPath: '/benchmark/',
    tab: 'benchmark',
    title: 'AI订阅额度实测对比 — ChatGPT Pro、Claude Max、Google AI Ultra 月 API 等值 | AI额度监控 QuotaReset',
    description:
      '对比 ChatGPT Plus/Pro、Claude Pro/Max/Ultra、Google AI Pro/Ultra 共 10 档订阅的平均月、每周、五小时 Token 容量与官方 API 等值、月费倍率及每 $1 Token，数据整理自 ModelDial 公开实测快照。',
    ogTitle: 'AI订阅额度实测对比 — ChatGPT、Claude、Google AI 各档月 API 等值',
    ogDescription:
      '10 档 AI 订阅的 Token 容量、官方 API 等值与月费倍率对比，覆盖 ChatGPT Pro 100/200/500、Claude Max/Ultra、Google AI Ultra 5×/20×。',
  },
];

const esc = (v) => v.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;');

function findBrowser() {
  const candidates = [
    process.env.BROWSER_BIN,
    '/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge',
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    '/Applications/Chromium.app/Contents/MacOS/Chromium',
    '/usr/bin/google-chrome',
    '/usr/bin/chromium',
    '/usr/bin/chromium-browser',
    '/usr/bin/microsoft-edge',
  ].filter(Boolean);
  return candidates.find((p) => fs.existsSync(p)) || null;
}

function sub(html, re, replacement, label) {
  if (!re.test(html)) throw new Error(`src/index.html 缺少可替换的 ${label}`);
  return html.replace(re, () => replacement);
}

function applyMeta(html, page) {
  html = sub(html, /<title>[^<]*<\/title>/, `<title>${esc(page.title)}</title>`, 'title');
  html = sub(html, /<meta name="description" content="[^"]*" \/>/, `<meta name="description" content="${esc(page.description)}" />`, 'description');
  html = sub(html, /<link rel="canonical" href="[^"]*" \/>/, `<link rel="canonical" href="${ORIGIN}${page.urlPath}" />`, 'canonical');
  html = sub(html, /<meta property="og:url" content="[^"]*" \/>/, `<meta property="og:url" content="${ORIGIN}${page.urlPath}" />`, 'og:url');
  html = sub(html, /<meta property="og:title" content="[^"]*" \/>/, `<meta property="og:title" content="${esc(page.ogTitle)}" />`, 'og:title');
  html = sub(html, /<meta property="og:description" content="[^"]*" \/>/, `<meta property="og:description" content="${esc(page.ogDescription)}" />`, 'og:description');
  html = sub(html, /<body data-initial-tab="\w+">/, `<body data-initial-tab="${page.tab}">`, 'body data-initial-tab');
  // JSON-LD 与 canonical 域名保持一致
  html = html.replace(/https:\/\/aireset\.xiangban\.xyz\//g, `${ORIGIN}/`);
  return html;
}

function prerender(browser, htmlPath, name) {
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), `aireset-profile-${name}-`));
  const r = spawnSync(
    browser,
    [
      '--headless=new',
      '--disable-gpu',
      '--no-first-run',
      '--no-default-browser-check',
      `--user-data-dir=${profile}`,
      '--virtual-time-budget=5000',
      '--dump-dom',
      `file://${htmlPath}`,
    ],
    { encoding: 'utf8', timeout: 90_000, maxBuffer: 64 * 1024 * 1024 }
  );
  fs.rmSync(profile, { recursive: true, force: true });
  if (r.status !== 0 || !r.stdout || r.stdout.length < 20_000) {
    throw new Error(`预渲染 ${name} 失败（exit=${r.status}）：${(r.stderr || '').slice(0, 400)}`);
  }
  let out = r.stdout;
  if (!/^\s*<!doctype html>/i.test(out)) out = '<!DOCTYPE html>\n' + out;
  return out;
}

function write(rel, content) {
  const target = path.join(DIST, rel);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, content);
}

function main() {
  const src = fs.readFileSync(SRC, 'utf8');
  fs.rmSync(DIST, { recursive: true, force: true });
  fs.mkdirSync(DIST, { recursive: true });

  const browser = NO_PRERENDER ? null : findBrowser();
  if (!NO_PRERENDER && !browser) {
    throw new Error('未找到 Chromium 内核浏览器用于预渲染；可设置 BROWSER_BIN，或加 --no-prerender（不利于 SEO，仅调试用）');
  }
  console.log(browser ? `预渲染浏览器：${browser}` : '跳过预渲染（--no-prerender）');

  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'aireset-src-'));
  for (const page of PAGES) {
    const html = applyMeta(src, page);
    let finalHtml = html;
    if (browser) {
      const tmpFile = path.join(tmp, `${page.tab}.html`);
      fs.writeFileSync(tmpFile, html);
      finalHtml = prerender(browser, tmpFile, page.tab);
    }
    write(page.out, finalHtml);
    console.log(`✓ ${page.urlPath}  ${(finalHtml.length / 1024).toFixed(0)} KB`);
  }
  fs.rmSync(tmp, { recursive: true, force: true });

  const today = new Date().toISOString().slice(0, 10);
  write(
    'favicon.svg',
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="8" fill="#0F172A"/><polyline points="29 16 24 16 20 27 12 5 8 16 3 16" transform="translate(0 0) scale(0.8) translate(4 4)" fill="none" stroke="#fff" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"/></svg>\n`
  );
  write('robots.txt', `User-agent: *\nAllow: /\n\nSitemap: ${ORIGIN}/sitemap.xml\n`);
  write(
    'sitemap.xml',
    `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n` +
      PAGES.map(
        (p) => `  <url>\n    <loc>${ORIGIN}${p.urlPath}</loc>\n    <lastmod>${today}</lastmod>\n    <changefreq>daily</changefreq>\n    <priority>${p.urlPath === '/' ? '1.0' : '0.8'}</priority>\n  </url>`
      ).join('\n') +
      `\n</urlset>\n`
  );
  write(
    '404.html',
    `<!DOCTYPE html>
<html lang="zh-CN"><head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1" />
<meta name="robots" content="noindex" /><title>页面不存在 — AI额度监控 QuotaReset</title>
<style>body{margin:0;min-height:100vh;display:grid;place-items:center;background:#F5F6F8;color:#0F172A;font:15px/1.6 -apple-system,"PingFang SC","Noto Sans SC",sans-serif}
main{text-align:center;padding:24px}h1{font-size:22px;margin:0 0 8px}p{margin:0 0 16px;color:#3E4C5E}
a{display:inline-block;padding:9px 16px;border-radius:8px;background:#1D4ED8;color:#fff;text-decoration:none;font-weight:600}</style></head>
<body><main><h1>页面不存在</h1><p>你访问的地址没有内容，可能已被移动。</p><a href="/">回到重置雷达</a></main></body></html>
`
  );
  console.log(`构建完成 → ${DIST}`);
}

main();
