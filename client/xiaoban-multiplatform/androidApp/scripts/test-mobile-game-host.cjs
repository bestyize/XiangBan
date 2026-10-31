// Exercises the production host's embedded JavaScript; no Android SDK or third-party packages.
// Run from the mobile project: node androidApp/scripts/test-mobile-game-host.cjs
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const source = fs.readFileSync(path.join(__dirname, '../src/main/kotlin/xyz/xiangban/xiaoban/MobileGameActivity.kt'), 'utf8');
const html = '<!doctype html>' + source.split('return """<!doctype html>')[1].split('""".trimIndent()')[0];
const template = html.match(/<script nonce="\$nonce">([\s\S]*?)<\/script>/)[1];
const init = { channel: 'xiaban.game-host', protocol: 1, type: 'host.init', nonce: 'test-nonce', save: null };
function script(saveEnabled = true) {
  return template
    .replaceAll('${JSONObject.quote(nonce)}', JSON.stringify('test-nonce'))
    .replaceAll('${JSONObject.quote(consoleToken)}', JSON.stringify('private-token:'))
    .replaceAll('${scriptJson(init.toString())}', JSON.stringify(init))
    .replaceAll('${game.saveEnabled}', String(saveEnabled))
    .replaceAll('${JSONObject.quote(gameUrl(game.id))}', JSON.stringify('https://xiaoban-game.invalid/games/test/index.html?hostNonce=test-nonce'));
}
function host(saveEnabled = true) {
  const logs = [], listeners = {}, frameListeners = {}, sent = [];
  const frameWindow = { postMessage: message => sent.push(message) };
  const frame = { contentWindow: frameWindow, addEventListener: (name, callback) => { frameListeners[name] = callback; } };
  const context = {
    document: { getElementById: () => frame },
    console: { log: message => logs.push(JSON.parse(message.slice('private-token:'.length))) },
    window: { addEventListener: (name, callback) => { listeners[name] = callback; } },
    TextEncoder, Date,
  };
  vm.runInNewContext(script(saveEnabled), context);
  frameListeners.load();
  const send = (message, origin = frameWindow) => listeners.message({ source: origin, data: {
    channel: 'xiaban.game-pack', protocol: 1, nonce: 'test-nonce', ...message,
  } });
  return { logs, sent, send, reload: frameListeners.load };
}

const game = host();
assert.equal(game.logs.at(-1).type, 'host.loaded');
assert.equal(game.sent[0].nonce, 'test-nonce');
let count = game.logs.length;
game.send({ type: 'game.save', data: { level: 1 } });
game.send({ type: 'game.start' });
game.send({ type: 'game.ready' }, {});
game.send({ type: 'game.ready', nonce: 'wrong' });
game.send({ type: 'game.ready', protocol: 2 });
game.send({ type: 'game.ready', channel: 'wrong' });
assert.equal(game.logs.length, count, 'source, nonce, protocol, channel and ready are required');

const noHandshake = host();
noHandshake.send({ type: 'game.exit' }, {});
noHandshake.send({ type: 'game.exit', nonce: 'wrong' });
assert.equal(noHandshake.logs.length, 1, 'even handshake-free exits require the frame source and nonce');
noHandshake.send({ type: 'game.exit' });
assert.equal(noHandshake.logs.at(-1).type, 'game.exit', 'games without protocol code can exit through the injected Escape handler');

game.send({ type: 'game.ready' });
assert.equal(game.logs.at(-1).type, 'game.ready');
count = game.logs.length;
game.send({ type: 'game.ready' });
game.send({ type: 'game.progress', score: 3 });
assert.equal(game.logs.length, count, 'duplicate ready and progress before start are ignored');
game.send({ type: 'game.start' });
game.send({ type: 'game.progress', score: 3 });
assert.equal(game.logs.at(-1).score, 3);
count = game.logs.length;
for (const score of [-1, 1.5, Infinity, NaN, '3', Number.MAX_SAFE_INTEGER + 1]) game.send({ type: 'game.result', score });
assert.equal(game.logs.length, count, 'scores must be safe nonnegative integers');
game.send({ type: 'game.result', score: 9 });
assert.equal(game.logs.at(-1).score, 9);
count = game.logs.length;
game.send({ type: 'game.result', score: 99 });
assert.equal(game.logs.length, count, 'one result per game.start');
game.send({ type: 'game.start' });
game.send({ type: 'game.result', score: 0 });
assert.equal(game.logs.at(-1).score, 0);

game.send({ type: 'game.save', data: { level: 2 } });
assert.equal(game.logs.at(-1).data.level, 2);
count = game.logs.length;
const cyclic = {}; cyclic.self = cyclic;
for (const data of [null, [], 'text', cyclic, { huge: 'x'.repeat(32768) }, { huge: '伴'.repeat(11000) }]) game.send({ type: 'game.save', data });
assert.equal(game.logs.length, count, 'saves are JSON objects limited to 32 KiB of UTF-8');

const withoutSave = host(false);
withoutSave.send({ type: 'game.ready' });
count = withoutSave.logs.length;
withoutSave.send({ type: 'game.save', data: {} });
assert.equal(withoutSave.logs.length, count, 'manifest save=false disables persistence');

game.send({ type: 'runtime.error', message: 'x'.repeat(2000), source: 'game.js', line: 2 });
assert.equal(game.logs.at(-1).type, 'runtime.error');
assert.equal(game.logs.at(-1).message.length, 1600);
for (let i = 0; i < 20; i++) game.send({ type: 'runtime.error', message: 'repeat' });
assert.equal(game.logs.filter(event => event.type === 'runtime.error').length, 5, 'error flood is bounded');
game.reload();
count = game.logs.length;
game.send({ type: 'game.save', data: {} });
assert.equal(game.logs.length, count, 'reloading requires a fresh ready handshake');

const bootstrap = source.split('val bootstrap = """<script>')[1].split('</script>""".trimIndent()')[0];
const childListeners = {}, childMessages = [];
vm.runInNewContext(bootstrap, {
  URLSearchParams, location: { search: '?hostNonce=test-nonce' },
  parent: { postMessage: message => childMessages.push(message) },
  window: { addEventListener: (name, callback) => { childListeners[name] = callback; } },
});
childListeners.error({ message: 'Syntax error', filename: 'game.js', lineno: 7 });
assert.equal(childMessages.at(-1).nonce, 'test-nonce');
assert.equal(childMessages.at(-1).line, 7);
childListeners.unhandledrejection({ reason: new Error('Promise failed') });
assert.equal(childMessages.at(-1).message, 'Promise failed');
childListeners.keydown({ key: 'Escape' });
assert.equal(childMessages.at(-1).type, 'game.exit');
console.log('Mobile game host protocol checks passed.');
