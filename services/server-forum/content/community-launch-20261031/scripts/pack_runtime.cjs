// Runtime resolution for resource rendering; no repository root is machine-specific.
const fs = require('fs'), path = require('path'), os = require('os');
function args() {
  const out = {};
  for (let i=2; i<process.argv.length; i+=2) {
    const flag=process.argv[i];
    if (!flag.startsWith('--') || process.argv[i+1]===undefined) throw new Error('Arguments must be --name value');
    out[flag.slice(2)] = process.argv[i+1];
  }
  const repo = path.resolve(out['repo-root'] || process.env.XB_REPO_ROOT || path.resolve(__dirname,'../../../../..'));
  return { repo, source: path.resolve(out['source-dir'] || process.env.XB_PACK_SOURCE_DIR || path.resolve(__dirname,'../packs')),
    output: path.resolve(out['output-dir'] || process.env.XB_PACK_BUILD_DIR || '/tmp/community-pack-rebuild'),
    base: out['base-url'] || process.env.XB_PACK_PREVIEW_ORIGIN || 'http://127.0.0.1:55138',
    playwright: out['playwright-module'] || process.env.XB_PLAYWRIGHT_MODULE,
    typescript: out['typescript-module'] || process.env.XB_TYPESCRIPT_MODULE,
    chromium: out.chromium || process.env.XB_CHROMIUM_EXECUTABLE };
}
function resolveModule(name, explicit, repo) {
  const candidates=[explicit, name];
  if (name==='typescript') candidates.push(path.join(repo,'services/server-forum/web/node_modules/typescript'),path.join(repo,'client/xiaoban-desktop/node_modules/typescript'));
  const runtime=process.env.CODEX_RUNTIME_NODE_MODULES || path.join(os.homedir(),'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules');
  candidates.push(path.join(runtime,name));
  for(const p of candidates.filter(Boolean)){try{return require(p)}catch(e){if(explicit===p)throw e}}
  throw Error(`Cannot resolve ${name}; pass --${name==='playwright'?'playwright':'typescript'}-module or configure CODEX_RUNTIME_NODE_MODULES`);
}
function chromiumOptions(pw, explicit) {
  if(explicit) return {executablePath:explicit,headless:true};
  // Prefer the runtime's normal installation; use the provided cached 1248 shell when present.
  const cache=process.env.PLAYWRIGHT_BROWSERS_PATH || path.join(os.homedir(),process.platform==='darwin'?'Library/Caches/ms-playwright':'.cache/ms-playwright');
  const shell=path.join(cache,'chromium_headless_shell-1248',process.platform==='darwin'?'chrome-headless-shell-mac-arm64/chrome-headless-shell':'chrome-headless-shell-linux64/chrome-headless-shell');
  if(fs.existsSync(shell)) return {executablePath:shell,headless:true};
  return {headless:true};
}
module.exports={args,resolveModule,chromiumOptions};
