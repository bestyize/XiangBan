import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const desktop = path.join(root, 'public/game-packs');
const reference = path.resolve(root, '../../assets/game-packs');
const errors = [];
// Deliberate desktop-only adaptations. Every desktop pack is self-contained, so
// the desktop index.html is derived from the reference pack by an exact, ordered rewrite
// (shared files and the share button are dropped, theme/host become local).
const desktopOnlyFiles = new Set(['desktop-host.js', 'theme.css']);
const freshThemePacks = new Set(['block-puzzle', 'merge-2048', 'water-sort']);
const shareButtonRemovedPacks = new Set(['falling-blocks', 'merge-2048', 'block-puzzle', 'water-sort']);
function desktopizeIndex(referenceText, id) {
  const lines = referenceText.split('\n').flatMap(line => {
    if (/\.\.\/result-share\.(?:css|js)/.test(line)) return [];
    if (shareButtonRemovedPacks.has(id) && line.includes('id="shareResultButton"')) return [];
    if (line.includes('href="../fresh-theme.css"')) return [line.replace('../fresh-theme.css', './theme.css')];
    return [line];
  });
  const text = lines.join('\n');
  return text.includes('<head>') ? text.replace('<head>', '<head><script src="./desktop-host.js"></script>') : null;
}
const desktopPackageAdaptations = {
  'falling-blocks/game.js': {
    reason: 'move result focus to restart when the unavailable desktop share action is hidden',
    normalize(text) {
      const desktopFocus = ";(new URLSearchParams(location.search).get('desktop') === '1' ? document.getElementById('restartOverlay') : document.getElementById('shareResultButton'))?.focus({ preventScroll: true })";
      const referenceFocus = "document.getElementById('shareResultButton')?.focus({ preventScroll: true })";
      return text.includes(desktopFocus) ? text.replace(desktopFocus, referenceFocus) : null;
    },
  },
};
// Top-level CSS rules, whitespace-normalized, to prove theme.css loses nothing from reference fresh-theme.css.
function cssRules(text) {
  const rules = [];
  let depth = 0, start = 0;
  const clean = text.replace(/\/\*[\s\S]*?\*\//g, '');
  for (let i = 0; i < clean.length; i++) {
    if (clean[i] === '{') depth++;
    else if (clean[i] === '}' && --depth === 0) { rules.push(clean.slice(start, i + 1).replace(/\s+/g, ' ').trim()); start = i + 1; }
  }
  return rules;
}
// The only rule desktop themes may add: the generic half of a reference media rule that also targets block-puzzle.
const splitThemeRules = new Set(['@media (max-height: 510px) { .fresh-game .quiet-button { min-height: 44px; } }']);
const readJson = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const exists = file => fs.existsSync(file) && fs.statSync(file).isFile();
const sourceText = file => fs.readFileSync(file, 'utf8').replace(/^\uFEFF/, '').replace(/\r\n?/g, '\n').replace(/\s+$/, '\n');
const localPath = (from, ref) => {
  const clean = ref.split(/[?#]/, 1)[0];
  if (!clean || clean.startsWith('#') || /^(data:|blob:|https?:|\/\/|javascript:|mailto:|tel:)/i.test(clean)) return null;
  return path.resolve(path.dirname(from), decodeURIComponent(clean.replace(/^\//, '')));
};
const within = (base, file) => file === base || file.startsWith(base + path.sep);
function checkRef(file, ref, base, label) {
  const target = localPath(file, ref);
  if (!target) return;
  if (!within(base, target)) { errors.push(`${label}: 引用越出 game-packs：${ref}`); return; }
  if (!exists(target)) errors.push(`${label}: 本地资源不存在：${ref}`);
}
function checkFile(file, base, label) {
  const text = fs.readFileSync(file, 'utf8');
  if (!/\.(?:html?|m?js|css)$/i.test(file)) return;
  if (/client[\\/]xiaban-reference[\\/]src|(?:\.\.[/\\])+src[/\\]/i.test(text)) errors.push(`${label}: 发现跨项目源码引用`);
  if (/\.html?$/i.test(file)) {
    for (const match of text.matchAll(/\b(?:src|href)\s*=\s*["']([^"']+)["']/gi)) checkRef(file, match[1], base, label);
  }
  if (/\.m?js$/i.test(file)) {
    for (const match of text.matchAll(/\bimport\s*(?:[^'";]*?\s+from\s*)?["']([^"']+)["']/g)) checkRef(file, match[1], base, label);
    for (const match of text.matchAll(/\bimport\s*\(\s*["']([^"']+)["']\s*\)/g)) checkRef(file, match[1], base, label);
  }
  if (/\.css$/i.test(file)) {
    for (const match of text.matchAll(/url\(\s*["']?([^"')]+)["']?\s*\)/gi)) checkRef(file, match[1].trim(), base, label);
    for (const match of text.matchAll(/@import\s+["']([^"']+)["']/gi)) checkRef(file, match[1], base, label);
  }
}
function walk(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap(item => {
    const file = path.join(dir, item.name);
    return item.isDirectory() ? walk(file) : [file];
  });
}
function filesUnder(dir) {
  return walk(dir).map(file => path.relative(dir, file).split(path.sep).join('/')).sort();
}
function normalizeWithAdaptation(text, adaptation, label) {
  if (!adaptation) return text;
  const normalized = adaptation.normalize(text);
  if (normalized === null) errors.push(`${label}: 登记的桌面适配差异缺失或已改变（${adaptation.reason}）`);
  return normalized;
}

for (const packRoot of [desktop, reference]) if (!fs.existsSync(packRoot)) errors.push(`游戏资源目录不存在：${packRoot}`);
if (!errors.length) {
  const desktopCatalog = readJson(path.join(desktop, 'catalog.json'));
  const referenceCatalog = readJson(path.join(reference, 'catalog.json'));
  if (desktopCatalog.version !== referenceCatalog.version || JSON.stringify(desktopCatalog.games) !== JSON.stringify(referenceCatalog.games)) errors.push('桌面 catalog 与基准 catalog 不一致');
  if (!Array.isArray(desktopCatalog.games) || desktopCatalog.games.length !== 7 || new Set(desktopCatalog.games).size !== 7) errors.push('catalog 必须列出七个唯一游戏包');
  for (const id of desktopCatalog.games) {
    for (const [base, side] of [[desktop, '桌面'], [reference, '基准']]) {
      const dir = path.join(base, id);
      const manifestFile = path.join(dir, 'manifest.json');
      if (!exists(manifestFile)) { errors.push(`${side}/${id}: 缺少 manifest.json`); continue; }
      let manifest;
      try { manifest = readJson(manifestFile); } catch { errors.push(`${side}/${id}: manifest.json 不是有效 JSON`); continue; }
      if (manifest.id !== id || manifest.runtimeProtocol !== 1) errors.push(`${side}/${id}: manifest id 或 runtimeProtocol 不匹配`);
      for (const key of ['entry', 'cover']) {
        if (typeof manifest[key] !== 'string' || !exists(path.resolve(dir, manifest[key]))) errors.push(`${side}/${id}: manifest ${key} 不存在`);
        else if (!within(dir, path.resolve(dir, manifest[key]))) errors.push(`${side}/${id}: manifest ${key} 越出游戏目录`);
      }
      if (side === '桌面') {
        for (const file of walk(dir)) checkFile(file, base, `${id}/${path.relative(dir, file)}`);
        const referenceManifestFile = path.join(reference, id, 'manifest.json');
        if (exists(referenceManifestFile) && sourceText(manifestFile) !== sourceText(referenceManifestFile)) errors.push(`${id}: 桌面与基准 manifest 不一致`);
      }
    }
    const desktopFiles = filesUnder(path.join(desktop, id));
    const referenceFiles = filesUnder(path.join(reference, id));
    for (const relative of desktopFiles) {
      if (desktopOnlyFiles.has(relative)) {
        if (relative === 'theme.css' && !freshThemePacks.has(id)) errors.push(`${id}: 不应存在 theme.css`);
        continue;
      }
      if (!referenceFiles.includes(relative)) errors.push(`${id}/${relative}: 桌面与基准资源清单不一致`);
    }
    for (const relative of referenceFiles) if (!desktopFiles.includes(relative)) errors.push(`${id}/${relative}: 桌面与基准资源清单不一致`);
    if (!desktopFiles.includes('desktop-host.js')) errors.push(`${id}: 缺少本地 desktop-host.js`);
    if (freshThemePacks.has(id) && !desktopFiles.includes('theme.css')) errors.push(`${id}: 缺少本地 theme.css`);
    for (const relative of referenceFiles) {
      const desktopFile = path.join(desktop, id, relative);
      const referenceFile = path.join(reference, id, relative);
      if (!exists(desktopFile) || !exists(referenceFile)) continue;
      const desktopText = sourceText(desktopFile);
      if (relative === 'index.html') {
        const expected = desktopizeIndex(sourceText(referenceFile), id);
        if (expected === null || expected !== desktopText) errors.push(`${id}/${relative}: 桌面入口页与基准入口页存在未登记的内容差异`);
        continue;
      }
      // README describes how this copy is packaged, which differs from the reference copy.
      if (relative === 'README.md') continue;
      const adaptation = desktopPackageAdaptations[`${id}/${relative}`];
      const normalized = normalizeWithAdaptation(desktopText, adaptation, `${id}/${relative}`);
      if (normalized !== sourceText(referenceFile)) errors.push(`${id}/${relative}: 桌面与基准资源存在未登记的内容差异`);
    }
  }
  const referenceThemeRules = cssRules(sourceText(path.join(reference, 'fresh-theme.css')));
  const themeRules = [...freshThemePacks].flatMap(id => exists(path.join(desktop, id, 'theme.css')) ? cssRules(sourceText(path.join(desktop, id, 'theme.css'))) : []);
  for (const rule of referenceThemeRules) if (!themeRules.includes(rule)) errors.push(`theme.css 丢失基准 fresh-theme.css 规则：${rule.slice(0, 60)}`);
  for (const rule of themeRules) if (!referenceThemeRules.includes(rule) && !splitThemeRules.has(rule)) errors.push(`theme.css 含基准 fresh-theme.css 之外的规则：${rule.slice(0, 60)}`);
  const hostSources = new Set(desktopCatalog.games.map(id => exists(path.join(desktop, id, 'desktop-host.js')) ? sourceText(path.join(desktop, id, 'desktop-host.js')) : ''));
  if (hostSources.size !== 1) errors.push('各游戏包的 desktop-host.js 副本不一致');
  for (const shared of ['desktop-host.js', 'fresh-theme.css', 'result-share.css', 'result-share.js']) if (exists(path.join(desktop, shared))) errors.push(`桌面 game-packs 根目录不应再有共享文件：${shared}`);
}
if (errors.length) { console.error(errors.map(error => `✗ ${error}`).join('\n')); process.exitCode = 1; }
else console.log('✓ 桌面七款游戏包、manifest、入口、封面、本地引用（自包含）及与基准版本的差异均通过只读校验。');
