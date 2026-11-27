import type { EvaluatedResult, PeerPayload, TestDefinition, TestId } from './types.ts';

export type RouteView =
  | { name: 'home' }
  | { name: 'quiz'; testId: TestId; peer?: PeerPayload }
  | { name: 'result'; testId: TestId; peer?: PeerPayload }
  | { name: 'compat'; testId: TestId; peer?: PeerPayload }
  | { name: 'archive' };

const VALID_IDS = new Set<TestId>([
  'sbti',
  'pantone',
  'animal',
  'manual',
  'checkup',
  'classic16'
]);

export function encodePeerShareUrl(
  baseUrl: string,
  test: TestDefinition,
  result: EvaluatedResult,
  nickname: string
): string {
  const vec = test.dimensions
    .slice(0, 4)
    .map((d) => Math.round(result.scores[d.id] ?? 50))
    .join('-');
  const params = new URLSearchParams();
  params.set('peer', result.profileCode);
  params.set('pn', nickname.trim().slice(0, 10) || '你的好友');
  params.set('v', vec);
  const cleanBase = baseUrl.split('#')[0];
  return `${cleanBase}#/t/${test.id}?${params.toString()}`;
}

export function decodePeerFromQuery(
  test: TestDefinition,
  query: URLSearchParams
): PeerPayload | undefined {
  const code = query.get('peer');
  if (!code) return undefined;
  const name = (query.get('pn') || '你的好友').slice(0, 12);
  const rawVec = (query.get('v') || '').split('-').map((s) => Number.parseInt(s, 10));
  const scores: Record<string, number> = {};
  test.dimensions.slice(0, 4).forEach((d, idx) => {
    const val = rawVec[idx];
    scores[d.id] = Number.isFinite(val) ? Math.max(5, Math.min(99, val)) : 50;
  });
  return {
    testId: test.id,
    profileCode: code,
    name,
    scores
  };
}

export function parseHashRoute(
  hash: string,
  findTest: (id: TestId) => TestDefinition | undefined
): RouteView {
  const clean = (hash || '').replace(/^#\/?/, '');
  if (!clean) return { name: 'home' };

  const [pathPart, queryPart = ''] = clean.split('?');
  const segs = pathPart.split('/').filter(Boolean);
  const query = new URLSearchParams(queryPart);

  if (segs[0] === 'archive') return { name: 'archive' };

  if ((segs[0] === 't' || segs[0] === 'result' || segs[0] === 'compat') && segs[1]) {
    const id = segs[1] as TestId;
    if (!VALID_IDS.has(id)) return { name: 'home' };
    const test = findTest(id);
    const peer = test ? decodePeerFromQuery(test, query) : undefined;
    if (segs[0] === 't') return { name: 'quiz', testId: id, peer };
    if (segs[0] === 'result') return { name: 'result', testId: id, peer };
    return { name: 'compat', testId: id, peer };
  }

  return { name: 'home' };
}

export function buildHashPath(view: RouteView): string {
  if (view.name === 'home') return '#/';
  if (view.name === 'archive') return '#/archive';
  const prefix =
    view.name === 'quiz' ? 't' : view.name === 'result' ? 'result' : 'compat';
  if (!view.peer) return `#/${prefix}/${view.testId}`;
  const params = new URLSearchParams();
  params.set('peer', view.peer.profileCode);
  params.set('pn', view.peer.name);
  const vals = Object.values(view.peer.scores)
    .slice(0, 4)
    .map((n) => Math.round(n))
    .join('-');
  if (vals) params.set('v', vals);
  return `#/${prefix}/${view.testId}?${params.toString()}`;
}
