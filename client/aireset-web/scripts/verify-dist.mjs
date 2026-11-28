#!/usr/bin/env node
/**
 * 构建产物自检（零依赖）：确认预渲染内容与 SEO 标签都已写进静态 HTML，且不含设计稿预览残留。
 * 用法：node scripts/verify-dist.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const DIST = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', 'dist');
const failures = [];
const check = (label, ok) => {
  console.log(`${ok ? '✓' : '✗'} ${label}`);
  if (!ok) failures.push(label);
};
const read = (rel) => (fs.existsSync(path.join(DIST, rel)) ? fs.readFileSync(path.join(DIST, rel), 'utf8') : '');
const count = (html, needle) => html.split(needle).length - 1;

const home = read('index.html');
const bench = read(path.join('benchmark', 'index.html'));

check('index.html 存在', home.length > 20_000);
check('benchmark/index.html 存在', bench.length > 20_000);

for (const [name, html, canonical] of [
  ['首页', home, 'https://aireset.xiangban.xyz/'],
  ['实测页', bench, 'https://aireset.xiangban.xyz/benchmark/'],
]) {
  check(`${name} canonical 正确`, html.includes(`<link rel="canonical" href="${canonical}"`));
  check(`${name} 含 description`, /<meta name="description" content="[^"]{40,}"/.test(html));
  check(`${name} 含 JSON-LD`, html.includes('application/ld+json'));
  check(`${name} 无设计稿预览残留`, !/viewportStage|specToolbar|spec-toolbar|data-vp=/.test(html));
  check(`${name} 无旧品牌名`, !/额度脉搏|QuotaPulse/.test(html));
}

check('首页预渲染了 13 条历史重置记录', count(home, '<td class="mono" style="white-space:nowrap">20') >= 13);
check('首页三张平台卡状态已生成', /data-status-chip="chatgpt"[^>]*>[^<]*全局重置/.test(home) || /status-chip (ok|warn|alert)"[^>]*data-status-chip="chatgpt"[^>]*>[^<]+</.test(home));
check('首页含演示数据声明', home.includes('演示数据'));
check('实测页预渲染了 10 档套餐行', count(bench, 'class="bench-row') >= 10);
check('实测页含 ModelDial 来源声明', bench.includes('modeldial.com/subscriptions'));
check('实测页 title 独立', /<title>AI订阅额度实测对比/.test(bench));
check('首页 title 独立', /<title>AI额度监控 QuotaReset — ChatGPT Codex/.test(home));

check('robots.txt 含 sitemap', read('robots.txt').includes('Sitemap: https://aireset.xiangban.xyz/sitemap.xml'));
check('sitemap.xml 含两个页面', count(read('sitemap.xml'), '<loc>') === 2);
check('favicon.svg 存在', read('favicon.svg').includes('<svg'));
check('404.html 存在', read('404.html').includes('页面不存在'));

if (failures.length) {
  console.error(`\n自检失败 ${failures.length} 项`);
  process.exit(1);
}
console.log('\n构建产物自检通过');
