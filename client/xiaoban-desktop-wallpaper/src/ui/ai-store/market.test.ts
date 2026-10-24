import { describe, expect, it } from 'vitest';
import { diffFiles, judgeLink, marketUnavailable, memberLabel, shortDigest, type MarketDetail } from './market.ts';

const sha = (c: string) => c.repeat(64);
const detail = (over: Partial<MarketDetail['item']> = {}, versions = [{ version: '1.2.0', yanked: false, manifestSha256: sha('a') }, { version: '1.1.0', yanked: false, manifestSha256: sha('b') }, { version: '1.0.0', yanked: true, manifestSha256: sha('c') }]): MarketDetail =>
  ({ item: { slug: 'pdf', name: 'PDF', description: '', namespace: 'acme', version: '1.2.0', ...over }, versions });

describe('web links are checked against the market before anything is reviewed', () => {
  it('accepts the latest version with its digest', () => {
    expect(judgeLink({ version: '1.2.0', sha256: sha('a') }, detail())).toEqual({ state: 'ok', version: '1.2.0', latest: '1.2.0' });
    expect(judgeLink({ version: '1.2.0', sha256: sha('A') }, detail()).state).toBe('ok');
  });
  it('offers the newest version when the link is older, but never silently', () => {
    expect(judgeLink({ version: '1.1.0', sha256: sha('b') }, detail())).toEqual({ state: 'newer', version: '1.2.0', latest: '1.2.0' });
  });
  it('refuses a digest that is not the market record', () => {
    expect(judgeLink({ version: '1.2.0', sha256: sha('f') }, detail())).toMatchObject({ state: 'mismatch', version: '1.2.0' });
  });
  it('refuses withdrawn or unknown versions', () => {
    expect(judgeLink({ version: '1.0.0', sha256: sha('c') }, detail()).state).toBe('missing');
    expect(judgeLink({ version: '9.9.9', sha256: null }, detail()).state).toBe('missing');
  });
  it('without a version, uses the latest one that is still available', () => {
    expect(judgeLink({ version: null, sha256: null }, detail())).toEqual({ state: 'ok', version: '1.2.0', latest: '1.2.0' });
    expect(judgeLink({ version: null, sha256: null }, detail({}, [])).state).toBe('missing');
  });
});

describe('update file differences', () => {
  const file = (path: string, hash: string, size = 10) => ({ path, size, sha256: hash, executableScript: path.startsWith('scripts/') });
  it('lists added, changed and removed files, additions first', () => {
    const changes = diffFiles(
      [{ path: 'SKILL.md', sha256: 'old' }, { path: 'references/legacy.md', sha256: 'x' }, { path: 'keep.md', sha256: 'same' }],
      [file('SKILL.md', 'new'), file('scripts/to_pdf.py', 'n'), file('keep.md', 'same')],
    );
    expect(changes.map(c => [c.change, c.path])).toEqual([['added', 'scripts/to_pdf.py'], ['changed', 'SKILL.md'], ['removed', 'references/legacy.md']]);
    expect(changes[0].executableScript).toBe(true);
  });
  it('reports nothing when the files are identical', () => {
    expect(diffFiles([{ path: 'SKILL.md', sha256: 'a' }], [file('SKILL.md', 'a')])).toEqual([]);
  });
});

describe('small helpers', () => {
  it('shortens digests for display only', () => {
    expect(shortDigest(sha('9'))).toBe('99999999…999999');
    expect(shortDigest('abc')).toBe('abc');
  });
  it('treats only connection failures as the market being down', () => {
    expect(marketUnavailable('暂时无法连接小伴市场')).toBe(true);
    expect(marketUnavailable('小伴市场暂不可用（HTTP 502）')).toBe(true);
    expect(marketUnavailable('条目不存在或未上架')).toBe(false);
    expect(marketUnavailable('小伴市场暂不可用（HTTP 404）')).toBe(false);
  });
  it('words team and bundle sizes differently', () => {
    expect(memberLabel('team', 4)).toBe('4 位专家');
    expect(memberLabel('bundle', 6)).toBe('6 个技能');
    expect(memberLabel('single', null)).toBe('');
  });
});
