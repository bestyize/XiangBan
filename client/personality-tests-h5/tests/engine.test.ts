import assert from 'node:assert/strict';
import test from 'node:test';
import { ALL_TESTS, buildSoulInsight, getTestById } from '../src/data/index.ts';
import { computeCompatibility, evaluateTest } from '../src/engine/scoring.ts';
import { encodePeerShareUrl, parseHashRoute } from '../src/engine/shareCodec.ts';
import { triggerSystemShare } from '../src/engine/systemShare.ts';

test('全部 6 套首发测评题库结构完整、无重复题号、无空选项', () => {
  assert.equal(ALL_TESTS.length, 6);
  for (const t of ALL_TESTS) {
    assert.ok(t.questions.length >= 16, `${t.id} 题量应不少于 16 题`);
    assert.ok(t.dimensions.length >= 4, `${t.id} 维度应不少于 4 个`);
    assert.ok(t.profiles.length >= 6, `${t.id} 结果原型应不少于 6 个`);
    const ids = new Set<string>();
    for (const q of t.questions) {
      assert.ok(!ids.has(q.id), `${t.id} 存在重复题号 ${q.id}`);
      ids.add(q.id);
      assert.equal(q.options.length, 4, `${q.id} 应有 4 个选项`);
    }
  }
});

test('16 型人格速测版 24 题可准确覆盖不同四字母编码', () => {
  const c16 = getTestById('classic16');
  assert.equal(c16.questions.length, 24);
  assert.equal(c16.profiles.length, 16);

  const allA: Record<string, string> = {};
  const allD: Record<string, string> = {};
  for (const q of c16.questions) {
    allA[q.id] = 'A';
    allD[q.id] = 'D';
  }
  const resA = evaluateTest(c16, allA);
  const resD = evaluateTest(c16, allD);
  assert.equal(resA.profileCode, 'ENFJ');
  assert.equal(resD.profileCode, 'ISTP');
});

test('SBTI 人类真实面具隐藏彩蛋与常规原型均可正常判定', () => {
  const sbti = getTestById('sbti');
  const maxChillGhost: Record<string, string> = {};
  for (const q of sbti.questions) {
    const best = [...q.options].sort(
      (a, b) =>
        (b.weights.chill ?? 0) +
        (b.weights.ghost ?? 0) -
        ((a.weights.chill ?? 0) + (a.weights.ghost ?? 0))
    )[0];
    maxChillGhost[q.id] = best.key;
  }
  const eggRes = evaluateTest(sbti, maxChillGhost);
  assert.equal(eggRes.profileCode, 'DRUNK');
  assert.equal(eggRes.isHiddenEgg, true);
});

test('零门槛双人合盘 URL 编解码与合拍报告计算完整闭环', () => {
  const animal = getTestById('animal');
  const myAnswers: Record<string, string> = {};
  const peerAnswers: Record<string, string> = {};
  animal.questions.forEach((q, idx) => {
    myAnswers[q.id] = 'A';
    peerAnswers[q.id] = idx % 2 === 0 ? 'A' : 'C';
  });

  const mine = evaluateTest(animal, myAnswers);
  const peerRes = evaluateTest(animal, peerAnswers);

  const shareUrl = encodePeerShareUrl('https://vendme.app/', animal, peerRes, '林小满');
  const hash = '#' + shareUrl.split('#')[1];
  const parsed = parseHashRoute(hash, getTestById);

  assert.equal(parsed.name, 'quiz');
  assert.equal(parsed.testId, 'animal');
  assert.ok(parsed.peer);
  assert.equal(parsed.peer.name, '林小满');
  assert.equal(parsed.peer.profileCode, peerRes.profileCode);

  const compat = computeCompatibility(animal, mine, parsed.peer);
  assert.ok(compat.score >= 66 && compat.score <= 98);
  assert.equal(compat.axes.length, 4);
  assert.equal(compat.synergies.length, 3);
  assert.equal(compat.cautions.length, 2);

  const insight = buildSoulInsight({
    animal: mine,
    sbti: evaluateTest(getTestById('sbti'), Object.fromEntries(getTestById('sbti').questions.map((q) => [q.id, 'A']))),
    pantone: evaluateTest(getTestById('pantone'), Object.fromEntries(getTestById('pantone').questions.map((q) => [q.id, 'B'])))
  });
  assert.ok(insight.includes('交叉验证'));
});

test('无 Web Share 环境调用系统分享时优雅降级不抛错', async () => {
  const outcome = await triggerSystemShare({
    title: '测试标题',
    text: '测试文案',
    url: 'https://example.com/#/t/sbti'
  });
  assert.ok(outcome.status === 'unsupported' || outcome.status === 'fallback-copied');
  assert.ok(outcome.message.length > 0);
});
