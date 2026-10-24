import { afterEach, describe, expect, it, vi } from 'vitest';
import { accountPasswordLogin, ApiError } from '../src/lib/api';

afterEach(() => vi.unstubAllGlobals());

const reply = (status: number, body: unknown) => vi.fn().mockResolvedValue(new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } }));

describe('accountPasswordLogin', () => {
  it('posts the credentials to the account center and returns the access token', async () => {
    const fetchMock = reply(200, { access_token: 'tok', refresh_token: 'r' });
    vi.stubGlobal('fetch', fetchMock);
    const token = await accountPasswordLogin('https://account.example/', 'xiaoban-market', ' alice ', 'pw');
    expect(token).toBe('tok');
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe('https://account.example/api/auth/password/login');
    expect(JSON.parse(init.body)).toMatchObject({ identifier: 'alice', password: 'pw', client_id: 'xiaoban-market' });
  });

  it('surfaces the account center message on failure', async () => {
    vi.stubGlobal('fetch', reply(401, { message: '账号或密码错误' }));
    await expect(accountPasswordLogin('https://a', 'c', 'x', 'y')).rejects.toMatchObject({ status: 401, message: '账号或密码错误' });
  });

  it('reports an unreachable account center clearly', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('network')));
    await expect(accountPasswordLogin('https://a', 'c', 'x', 'y')).rejects.toBeInstanceOf(ApiError);
    await expect(accountPasswordLogin('https://a', 'c', 'x', 'y')).rejects.toThrow('无法连接账号中心');
  });
});
