// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import AiGatewayCard from './AiGatewayCard.vue';
import type { ControlStore } from '../control-types.ts';
import { AccountError, invoke, type AiBalance, type AiConfigView, type AiTier, type AccountSession } from '../../bridge.ts';

vi.mock('../../bridge.ts', async importOriginal => ({ ...await importOriginal<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
const ipc = vi.mocked(invoke);

const session = (uid: string): AccountSession => ({ accessToken: 'a', refreshToken: 'r', expiresAt: 1, user: { uid, username: uid } });

const trialOnly: AiBalance = {
  product: { name: '小伴壁纸' },
  quotaPerUnit: 500_000,
  total: 250_000,
  paid: 0,
  buckets: [{ id: 1, kind: 'trial', total: 250_000, remaining: 250_000, expiresAt: '2099-01-01T00:00:00Z', createdAt: '2026-10-18T00:00:00Z' }],
  trial: { available: false, quota: 250_000, days: 7 },
};
const fresh: AiBalance = { ...trialOnly, total: 0, buckets: [], trial: { available: true, quota: 250_000, days: 7 } };
const afterMonthly: AiBalance = {
  ...trialOnly,
  total: 2_750_000,
  paid: 2_500_000,
  buckets: [
    ...trialOnly.buckets,
    { id: 2, kind: 'monthly', total: 2_500_000, remaining: 2_500_000, expiresAt: '2099-02-01T00:00:00Z', createdAt: '2026-10-18T00:00:00Z' },
  ],
};

const standard: AiTier = { tier: 'standard', displayName: '日常', description: '响应更快，额度更省', isDefault: true, requiresPaid: false, available: true };
const premium: AiTier = { tier: 'premium', displayName: '深入', description: '适合复杂任务，耗时和额度更多', isDefault: false, requiresPaid: true, available: false };
const premiumOpen: AiTier = { ...premium, available: true };

const aiConfig = (patch: Partial<AiConfigView> = {}): AiConfigView => ({
  activeProvider: 'deepseek', providers: [], protocol: 'responses', baseUrl: 'https://api.deepseek.com', model: 'm', hasKey: true, keyHint: '••••',
  workspace: '', defaultWorkspace: '/w', hideToolCalls: false, openComputerUse: false, ...patch,
});

type Handler = unknown | ((args?: Record<string, unknown>) => unknown);
function backend(handlers: Record<string, Handler>) {
  ipc.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
    const handler = handlers[command];
    if (handler === undefined) throw new Error(`unmocked ${command}`);
    const value = typeof handler === 'function' ? (handler as (a?: Record<string, unknown>) => unknown)(args) : handler;
    if (value instanceof Error) throw value;
    return value;
  });
}
const defaults = { ai_gateway_balance: trialOnly, ai_gateway_tiers: [standard, premium] };

const createStore = (account: AccountSession | null, desktop = true) =>
  reactive({ desktop, account, notify: vi.fn(), openLogin: vi.fn() }) as unknown as ControlStore;
let wrapper: VueWrapper | undefined;
const mountCard = (store: ControlStore, config: AiConfigView | null = aiConfig()) => (wrapper = mount(AiGatewayCard, { props: { store, config }, attachTo: document.body }));

// The dialog is teleported to <body>, the card itself is not.
const q = <T extends HTMLElement = HTMLElement>(testid: string) => document.body.querySelector<T>(`[data-testid="${testid}"]`);
const text = (testid: string) => q(testid)?.textContent?.trim() ?? '';
const has = (testid: string) => q(testid) !== null;
const click = async (testid: string) => {
  const el = q(testid);
  if (!el) throw new Error(`missing ${testid}`);
  el.click();
  await flushPromises();
};
const typeCode = async (value: string) => {
  const input = q<HTMLInputElement>('gateway-code')!.querySelector('input')!;
  input.value = value;
  input.dispatchEvent(new Event('input'));
  await flushPromises();
};
const submit = async () => {
  q('gateway-redeem')!.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true }));
  await flushPromises();
};
const codeValue = () => q<HTMLInputElement>('gateway-code')!.querySelector('input')!.value;
const gatewayCalls = () => ipc.mock.calls.filter(([command]) => command.startsWith('ai_gateway_') || command === 'use_ai_gateway');
const openDialog = async () => { await click('gateway-open'); };

// A hook that returns a function is treated as its cleanup callback, so don't return the mock.
beforeEach(() => { ipc.mockReset(); });
afterEach(() => { wrapper?.unmount(); wrapper = undefined; document.body.innerHTML = ''; });

describe('小伴 AI card', () => {
  it('invites a signed-out user to log in without calling the gateway', async () => {
    backend(defaults);
    const store = createStore(null);
    mountCard(store);
    await flushPromises();
    expect(text('gateway-card-status')).toBe('需登录');
    expect(wrapper!.find('[data-testid="gateway-open"]').text()).toBe('配置');
    expect(wrapper!.find('[data-testid="gateway-activate"]').text()).toBe('启用此服务商');
    expect(has('gateway-modal')).toBe(false);

    await openDialog();
    expect(has('gateway-signin')).toBe(true);
    expect(has('gateway-redeem')).toBe(false);
    expect(has('gateway-use')).toBe(false);
    await click('gateway-login');
    expect(store.openLogin).toHaveBeenCalled();
    expect(gatewayCalls()).toHaveLength(0);
  });

  it('shows the balance on the card and keeps everything else in the dialog', async () => {
    backend({ ...defaults, ai_gateway_balance: afterMonthly });
    mountCard(createStore(session('u1')));
    await flushPromises();
    expect(text('gateway-card-status')).toBe('可使用');
    expect(text('gateway-card-fact')).toBe('$5.50');
    expect(has('gateway-code')).toBe(false);
    expect(has('gateway-tier-standard')).toBe(false);
  });

  it('says which tier is in use on the card and highlights it', async () => {
    backend({ ...defaults, ai_gateway_tiers: [standard, premiumOpen] });
    mountCard(createStore(session('u1')), aiConfig({ activeProvider: 'xiaoban-ai', gatewayTier: 'premium' }));
    await flushPromises();
    expect(text('gateway-card-status')).toBe('当前使用');
    expect(text('gateway-card-fact')).toBe('深入');
    expect(wrapper!.find('[data-testid="ai-gateway-card"]').classes()).toContain('active');
  });

  it('does not call anything outside the desktop app', async () => {
    backend(defaults);
    mountCard(createStore(session('u1'), false));
    await flushPromises();
    expect(ipc).not.toHaveBeenCalled();
    expect(wrapper!.find('[data-testid="gateway-open"]').attributes('disabled')).toBeDefined();
    expect(wrapper!.find('[data-testid="gateway-activate"]').attributes('disabled')).toBeDefined();
  });

  it('shows the same two buttons as the other providers, and the card itself is not a button', async () => {
    backend(defaults);
    mountCard(createStore(session('u1')));
    await flushPromises();
    const buttons = wrapper!.findAll('.gw-actions button').map(b => b.text());
    expect(buttons).toEqual(['配置', '启用此服务商']);
    await wrapper!.find('[data-testid="ai-gateway-card"]').trigger('click');
    await flushPromises();
    expect(has('gateway-modal')).toBe(false);
  });

  it('opens the dialog with a fresh load when the card is clicked', async () => {
    backend({ ...defaults, ai_gateway_balance: afterMonthly });
    mountCard(createStore(session('u1')));
    await flushPromises();
    const before = gatewayCalls().length;
    await openDialog();
    expect(gatewayCalls().length).toBeGreaterThan(before);
    expect(text('gateway-total')).toBe('$5.50');
    expect(text('gateway-paid')).toContain('$5.00');
    const rows = document.body.querySelectorAll('[data-testid="gateway-buckets"] li');
    expect(rows).toHaveLength(2);
    expect(rows[0].textContent).toContain('试用额度');
    expect(rows[1].textContent).toContain('月卡');
    expect(text('gateway-subtitle')).toContain('小伴壁纸');
  });

  it('drops the old account\'s balance and tiers when another account signs in', async () => {
    let owner = 'u1';
    backend({ ...defaults, ai_gateway_balance: () => (owner === 'u1' ? afterMonthly : fresh), ai_gateway_tiers: () => (owner === 'u1' ? [standard, premiumOpen] : [standard, premium]) });
    const store = createStore(session('u1'));
    mountCard(store);
    await flushPromises();
    await openDialog();
    expect(text('gateway-total')).toBe('$5.50');
    expect(q('gateway-tier-premium')!.hasAttribute('disabled')).toBe(false);

    owner = 'u2';
    store.account = session('u2');
    await flushPromises();
    expect(text('gateway-total')).toBe('$0.00');
    expect(q('gateway-tier-premium')!.hasAttribute('disabled')).toBe(true);
  });
});

describe('小伴 AI dialog: balance and redeem', () => {
  it('tells a new account about the trial instead of showing an empty list', async () => {
    backend({ ...defaults, ai_gateway_balance: fresh });
    mountCard(createStore(session('u1')));
    await flushPromises();
    await openDialog();
    expect(text('gateway-trial-hint')).toContain('$0.50 试用额度');
    expect(text('gateway-trial-hint')).toContain('7 天');
  });

  it('keeps the redeem button off until something is typed, then redeems and reloads the balance', async () => {
    let balance = trialOnly;
    backend({
      ...defaults,
      ai_gateway_balance: () => balance,
      ai_gateway_redeem: () => { balance = afterMonthly; return { kind: 'monthly', quota: 2_500_000, expiresAt: '2099-02-01T00:00:00Z', balance: 2_750_000 }; },
    });
    mountCard(createStore(session('u1')));
    await flushPromises();
    await openDialog();
    expect(q('gateway-redeem')!.hasAttribute('disabled')).toBe(true);

    await typeCode('  ABCD-EFGH  ');
    expect(q('gateway-redeem')!.hasAttribute('disabled')).toBe(false);
    await submit();

    expect(ipc).toHaveBeenCalledWith('ai_gateway_redeem', { code: '  ABCD-EFGH  ' });
    expect(text('gateway-redeem-done')).toContain('$5.00');
    expect(text('gateway-total')).toBe('$5.50');
    expect(text('gateway-card-fact')).toBe('$5.50');
    expect(codeValue()).toBe('');
  });

  it('shows the gateway message when a code is rejected and keeps the code for another try', async () => {
    backend({ ...defaults, ai_gateway_redeem: new AccountError('兑换码无效或已被使用', 'code_invalid') });
    mountCard(createStore(session('u1')));
    await flushPromises();
    await openDialog();
    await typeCode('WRONG');
    await submit();
    expect(text('gateway-redeem-error')).toBe('兑换码无效或已被使用');
    expect(has('gateway-redeem-done')).toBe(false);
    expect(codeValue()).toBe('WRONG');
  });

  it('reports a failed load without hiding the redeem box', async () => {
    backend({ ...defaults, ai_gateway_balance: () => { throw { message: '小伴 AI 服务暂时不可用，请稍后重试' }; } });
    mountCard(createStore(session('u1')));
    await flushPromises();
    await openDialog();
    expect(text('gateway-load-error')).toContain('暂时不可用');
    expect(has('gateway-total')).toBe(false);
    expect(has('gateway-redeem')).toBe(true);
  });
});

describe('小伴 AI dialog: chat model', () => {
  it('lists the tiers, preselects the default and locks a paid tier the account cannot use yet', async () => {
    backend(defaults);
    mountCard(createStore(session('u1')));
    await flushPromises();
    await openDialog();
    expect(q('gateway-tier-standard')!.getAttribute('aria-checked')).toBe('true');
    expect(q('gateway-tier-premium')!.getAttribute('aria-checked')).toBe('false');
    expect(q('gateway-tier-premium')!.hasAttribute('disabled')).toBe(true);
    expect(text('gateway-tier-premium')).toContain('兑换付费额度后可用');
    expect(text('gateway-use-btn')).toBe('用小伴 AI 聊天');
    expect(has('gateway-active')).toBe(false);
  });

  it('switches chat to the picked tier and hands the new settings to the parent', async () => {
    backend({
      ...defaults,
      ai_gateway_tiers: [standard, premiumOpen],
      use_ai_gateway: (args?: Record<string, unknown>) => aiConfig({ activeProvider: 'xiaoban-ai', gatewayTier: String(args?.tier) }),
    });
    const store = createStore(session('u1'));
    mountCard(store);
    await flushPromises();
    await openDialog();
    await click('gateway-tier-premium');
    expect(q('gateway-tier-premium')!.getAttribute('aria-checked')).toBe('true');
    await click('gateway-use-btn');

    expect(ipc).toHaveBeenCalledWith('use_ai_gateway', { tier: 'premium' });
    const emitted = wrapper!.emitted('config');
    expect(emitted).toHaveLength(1);
    expect((emitted![0][0] as AiConfigView).gatewayTier).toBe('premium');
    expect(store.notify).toHaveBeenCalledWith(expect.stringContaining('深入'), 'success');

    await wrapper!.setProps({ config: emitted![0][0] as AiConfigView });
    await flushPromises();
    expect(text('gateway-active')).toContain('深入');
    expect(text('gateway-use-btn')).toBe('当前使用中');
    expect(q('gateway-use-btn')!.hasAttribute('disabled')).toBe(true);
    expect(text('gateway-card-status')).toBe('当前使用');
  });

  it('starts from the tier already in use and offers to switch when another one is picked', async () => {
    backend({ ...defaults, ai_gateway_tiers: [standard, premiumOpen] });
    mountCard(createStore(session('u1')), aiConfig({ activeProvider: 'xiaoban-ai', gatewayTier: 'premium' }));
    await flushPromises();
    await openDialog();
    expect(text('gateway-active')).toContain('深入');
    expect(q('gateway-tier-premium')!.getAttribute('aria-checked')).toBe('true');
    expect(q('gateway-use-btn')!.hasAttribute('disabled')).toBe(true);

    await click('gateway-tier-standard');
    expect(text('gateway-use-btn')).toBe('切换模式');
    expect(q('gateway-use-btn')!.hasAttribute('disabled')).toBe(false);
  });

  it('does not claim the gateway is in use while another provider is', async () => {
    backend(defaults);
    mountCard(createStore(session('u1')), aiConfig({ activeProvider: 'deepseek', gatewayTier: 'standard' }));
    await flushPromises();
    await openDialog();
    expect(has('gateway-active')).toBe(false);
    expect(text('gateway-card-status')).toBe('可使用');
    expect(text('gateway-use-btn')).toBe('用小伴 AI 聊天');
  });

  it('unlocks the paid tier once a code has been redeemed', async () => {
    let paid = false;
    backend({
      ...defaults,
      ai_gateway_tiers: () => [standard, paid ? premiumOpen : premium],
      ai_gateway_redeem: () => { paid = true; return { kind: 'monthly', quota: 2_500_000, expiresAt: null, balance: 2_750_000 }; },
    });
    mountCard(createStore(session('u1')));
    await flushPromises();
    await openDialog();
    expect(q('gateway-tier-premium')!.hasAttribute('disabled')).toBe(true);
    await typeCode('PAID-CODE');
    await submit();
    expect(q('gateway-tier-premium')!.hasAttribute('disabled')).toBe(false);
  });

  it('shows why switching failed and leaves the button usable', async () => {
    backend({ ...defaults, use_ai_gateway: new AccountError('无法保存模型配置：磁盘已满') });
    mountCard(createStore(session('u1')));
    await flushPromises();
    await openDialog();
    await click('gateway-use-btn');
    expect(text('gateway-use-error')).toContain('磁盘已满');
    expect(q('gateway-use-btn')!.hasAttribute('disabled')).toBe(false);
    expect(wrapper!.emitted('config')).toBeUndefined();
  });

  it('keeps the balance when only the tier list fails', async () => {
    backend({ ...defaults, ai_gateway_tiers: () => { throw { message: '小伴 AI 服务暂时不可用，请稍后重试' }; } });
    mountCard(createStore(session('u1')));
    await flushPromises();
    await openDialog();
    expect(text('gateway-total')).toBe('$0.50');
    expect(has('gateway-use')).toBe(false);
    expect(text('gateway-load-error')).toContain('暂时不可用');
  });
});

describe('小伴 AI card: 启用此服务商', () => {
  const useCalls = () => ipc.mock.calls.filter(([command]) => command === 'use_ai_gateway');
  const withUse = (extra: Record<string, Handler> = {}) => backend({
    ...defaults,
    use_ai_gateway: (args?: Record<string, unknown>) => aiConfig({ activeProvider: 'xiaoban-ai', gatewayTier: String(args?.tier) }),
    ...extra,
  });

  it('turns chat to the preselected tier straight from the card, without opening the dialog', async () => {
    withUse();
    const store = createStore(session('u1'));
    mountCard(store);
    await flushPromises();
    await wrapper!.find('[data-testid="gateway-activate"]').trigger('click');
    await flushPromises();

    expect(useCalls()).toEqual([['use_ai_gateway', { tier: 'standard' }]]);
    expect(has('gateway-modal')).toBe(false);
    const emitted = wrapper!.emitted('config');
    expect(emitted).toHaveLength(1);
    expect((emitted![0][0] as AiConfigView).activeProvider).toBe('xiaoban-ai');
    expect(store.notify).toHaveBeenCalledWith(expect.stringContaining('日常'), 'success');
  });

  it('becomes a disabled "正在使用中" once it is the provider in use, like the other cards', async () => {
    withUse();
    mountCard(createStore(session('u1')), aiConfig({ activeProvider: 'xiaoban-ai', gatewayTier: 'standard' }));
    await flushPromises();
    expect(has('gateway-activate')).toBe(false);
    expect(wrapper!.find('[data-testid="gateway-in-use"]').text()).toBe('正在使用中');
    expect(wrapper!.find('[data-testid="gateway-in-use"]').attributes('disabled')).toBeDefined();
    expect(wrapper!.find('[data-testid="gateway-open"]').text()).toBe('配置');
  });

  it('asks a signed-out user to log in instead, and opens the dialog that has the login button', async () => {
    withUse();
    const store = createStore(null);
    mountCard(store);
    await flushPromises();
    await wrapper!.find('[data-testid="gateway-activate"]').trigger('click');
    await flushPromises();
    expect(useCalls()).toHaveLength(0);
    expect(store.notify).toHaveBeenCalledWith(expect.stringContaining('登录'), 'info');
    expect(has('gateway-signin')).toBe(true);
  });

  it('opens the dialog with the reason when there is no tier to use yet', async () => {
    withUse({ ai_gateway_tiers: [] });
    const store = createStore(session('u1'));
    mountCard(store);
    await flushPromises();
    await wrapper!.find('[data-testid="gateway-activate"]').trigger('click');
    await flushPromises();
    expect(useCalls()).toHaveLength(0);
    expect(store.notify).toHaveBeenCalledWith(expect.stringContaining('没有可用的模式'), 'info');
    expect(has('gateway-modal') || has('gateway-redeem')).toBe(true);
  });

  it('loads the tiers first when they have not arrived by the time the button is pressed', async () => {
    let release!: () => void;
    const gate = new Promise<void>(resolve => { release = resolve; });
    withUse({ ai_gateway_tiers: async () => { await gate; return [standard, premium]; } });
    mountCard(createStore(session('u1')));
    await flushPromises();
    const pending = wrapper!.find('[data-testid="gateway-activate"]').trigger('click');
    release();
    await pending;
    await flushPromises();
    expect(useCalls()).toEqual([['use_ai_gateway', { tier: 'standard' }]]);
  });

  it('tells the user why it failed when the dialog is closed, and the button works again', async () => {
    withUse({ use_ai_gateway: new AccountError('小伴 AI 入口已被关闭，暂时不能启用') });
    const store = createStore(session('u1'));
    mountCard(store);
    await flushPromises();
    await wrapper!.find('[data-testid="gateway-activate"]').trigger('click');
    await flushPromises();
    expect(store.notify).toHaveBeenCalledWith('小伴 AI 入口已被关闭，暂时不能启用', 'error');
    expect(wrapper!.find('[data-testid="gateway-activate"]').attributes('disabled')).toBeUndefined();
    expect(wrapper!.emitted('config')).toBeUndefined();
  });

  it('shows a failure in the dialog instead of a toast when it is the dialog that was used', async () => {
    withUse({ use_ai_gateway: new AccountError('磁盘已满') });
    const store = createStore(session('u1'));
    mountCard(store);
    await flushPromises();
    await openDialog();
    await click('gateway-use-btn');
    expect(text('gateway-use-error')).toContain('磁盘已满');
    expect(store.notify).not.toHaveBeenCalledWith('磁盘已满', 'error');
  });
});
