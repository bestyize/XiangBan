const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const { test } = require('node:test');
const vm = require('node:vm');

const script = readFileSync(join(__dirname, '../src/main/assets/runtime-v2/bridge.js'), 'utf8')
  .replace('__XB_INITIAL_INFO__', '{}')
  .replace('__XB_CAPABILITIES__', '[]');

function bridge() {
  const calls = [];
  const sandbox = {
    document: { documentElement: { style: { setProperty() {} } } },
    addEventListener() {},
    setTimeout,
    clearTimeout,
    xbNative: { postMessage: payload => calls.push(JSON.parse(payload)) },
  };
  sandbox.window = sandbox.top = sandbox;
  vm.runInNewContext(script, sandbox);
  function reply(method, ok = true) {
    const request = calls.find(call => call.method === method && !call.replied);
    assert(request, method);
    request.replied = true;
    sandbox.xbNative.onmessage({ data: JSON.stringify({
      channel: 'xiaban.host', protocol: 2, kind: 'result', id: request.id,
      ok, ...(ok ? {} : { error: { code: 'invalid' } }),
    }) });
  }
  reply('document.connect');
  return { xb: sandbox.xb, calls, reply };
}

const nextTurn = () => new Promise(resolve => setImmediate(resolve));

test('the first round waits for readiness and its reports wait for the opening receipt', async () => {
  const { xb, calls, reply } = bridge();
  const start = xb.game.start();
  assert.equal(xb.game.start(), start, 'duplicate deferred starts share a receipt');
  const progress = xb.game.progress({ score: 30 });
  const result = xb.game.result({ score: 810 });
  const ready = xb.ready();
  assert.deepEqual(calls.map(call => call.method), ['document.connect', 'ready']);
  reply('ready');
  await ready;
  await nextTurn();
  assert.deepEqual(calls.map(call => call.method), ['document.connect', 'ready', 'game.start']);
  reply('game.start');
  await start;
  await nextTurn();
  assert.deepEqual(calls.slice(3).map(call => [call.method, call.params.score]), [
    ['game.progress', 30], ['game.result', 810],
  ]);
  reply('game.progress');
  reply('game.result');
  await Promise.all([progress, result]);

  const restart = xb.game.start();
  await nextTurn();
  reply('game.start');
  await restart;
  assert.equal(calls.filter(call => call.method === 'game.start').length, 2);
});

test('a failed readiness acknowledgement rejects the queued start without posting it', async () => {
  const { xb, calls, reply } = bridge();
  const start = xb.game.start();
  const ready = xb.ready();
  reply('ready', false);
  await Promise.all([
    assert.rejects(ready, { code: 'invalid' }),
    assert.rejects(start, { code: 'invalid' }),
  ]);
  assert.deepEqual(calls.map(call => call.method), ['document.connect', 'ready']);
});
