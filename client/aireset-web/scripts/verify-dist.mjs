#!/usr/bin/env node
/**
 * 构建产物自检（零依赖）：确认中英文 4 个预渲染页面与 SEO 标签均正确写入静态 HTML，且无旧品牌或预览残留。
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

const homeZh = read('index.html');
const benchZh = read(path.join('benchmark', 'index.html'));
const homeEn = read(path.join('en', 'index.html'));
const benchEn = read(path.join('en', 'benchmark', 'index.html'));

const pages = [
  ['中文首页 /', homeZh, 'https://aireset.xiaban.xyz/', 'https://aireset.xiaban.xyz/', 'https://aireset.xiaban.xyz/en/', 'zh-CN'],
  ['中文实测 /benchmark/', benchZh, 'https://aireset.xiaban.xyz/benchmark/', 'https://aireset.xiaban.xyz/benchmark/', 'https://aireset.xiaban.xyz/en/benchmark/', 'zh-CN'],
  ['英文首页 /en/', homeEn, 'https://aireset.xiaban.xyz/en/', 'https://aireset.xiaban.xyz/', 'https://aireset.xiaban.xyz/en/', 'en'],
  ['英文实测 /en/benchmark/', benchEn, 'https://aireset.xiaban.xyz/en/benchmark/', 'https://aireset.xiaban.xyz/benchmark/', 'https://aireset.xiaban.xyz/en/benchmark/', 'en'],
];

for (const [name, html, canonical, hrefZh, hrefEn, htmlLang] of pages) {
  check(`${name} 文件存在`, html.length > 20_000);
  check(`${name} html lang="${htmlLang}"`, html.includes(`<html lang="${htmlLang}"`));
  check(`${name} canonical 正确`, html.includes(`<link rel="canonical" href="${canonical}"`));
  check(`${name} hreflang zh-CN 正确`, html.includes(`<link rel="alternate" hreflang="zh-CN" href="${hrefZh}"`));
  check(`${name} hreflang en 正确`, html.includes(`<link rel="alternate" hreflang="en" href="${hrefEn}"`));
  check(`${name} 含 description`, /<meta name="description" content="[^"]{40,}"/.test(html));
  check(`${name} 含 JSON-LD`, html.includes('application/ld+json'));
  check(`${name} 无设计稿预览残留`, !/viewportStage|specToolbar|spec-toolbar|data-vp=/.test(html));
  check(`${name} 无旧品牌名`, !/AI额度监控|额度脉搏|QuotaPulse/.test(html));
}

// 中文页校验
check('中文首页品牌为 AI订阅测评', homeZh.includes('AI订阅测评'));
check('中文首页 title 独立', /<title>AI订阅测评 QuotaReset — ChatGPT Codex/.test(homeZh));
check('中文首页预渲染 13 条历史记录', count(homeZh, '<td class="mono" style="white-space:nowrap">20') >= 13);
check('中文首页平台状态标签已生成', /status-chip (ok|warn|alert)" data-status-chip="chatgpt"[^>]*>[^<]+</.test(homeZh));
check('中文首页含演示数据声明', homeZh.includes('演示数据'));
check('中文实测页 title 独立', /<title>AI订阅额度实测对比/.test(benchZh));
check('中文实测页预渲染 10 档套餐', count(benchZh, 'class="bench-row') >= 10);
check('中文实测页含 ModelDial 来源声明', benchZh.includes('modeldial.com/subscriptions'));

// 英文页校验
check('英文首页 title 为英文', /<title>QuotaReset — ChatGPT Codex, Google Antigravity/.test(homeEn));
check('英文首页 h1 为英文', homeEn.includes('Big-Three Reset Radar'));
check('英文首页预渲染 13 条历史记录', count(homeEn, '<td class="mono" style="white-space:nowrap">20') >= 13);
check('英文首页含英文演示数据声明', homeEn.includes('demo data'));
check('英文实测页 title 为英文', /<title>AI Subscription Quota Benchmarks/.test(benchEn));
check('英文实测页预渲染 10 档套餐', count(benchEn, 'class="bench-row') >= 10);
check('英文实测页使用 B/M 单位 (2.9B / 145M)', benchEn.includes('2.9B') && benchEn.includes('145M'));
check('英文实测页含 ModelDial 来源声明', benchEn.includes('modeldial.com/subscriptions'));

// 静态辅助文件
check('robots.txt 含 sitemap', read('robots.txt').includes('Sitemap: https://aireset.xiaban.xyz/sitemap.xml'));
check('sitemap.xml 含 4 个中英文页面', count(read('sitemap.xml'), '<loc>') === 4);
check('favicon.svg 存在', read('favicon.svg').includes('<svg'));
check('404.html 含中英文双语链接', read('404.html').includes('/en/') && read('404.html').includes('Page Not Found'));

if (failures.length) {
  console.error(`\n自检失败 ${failures.length} 项`);
  process.exit(1);
}
console.log('\n构建产物自检通过');
