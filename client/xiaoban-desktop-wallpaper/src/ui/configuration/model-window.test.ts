// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { reactive } from 'vue';
import AiConfigCard from '../components/AiConfigCard.vue';
import AiGatewayCard from '../components/AiGatewayCard.vue';
import { invoke, listen, type AiConfigView } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
vi.mock('../../bridge.ts', async original => ({ ...await original<typeof import('../../bridge.ts')>(), desktop: true, invoke: vi.fn(), listen: vi.fn() }));
const ipc = vi.mocked(invoke);
const view: AiConfigView = { activeProvider: 'deepseek', providers: [{ id: 'deepseek', name: 'DeepSeek', protocol: 'chat', baseUrl: 'https://api.example.com/v1', model: 'model-a', hasKey: true, keyHint: '••••1234', isCustom: false }], protocol: 'chat', baseUrl: 'https://api.example.com/v1', model: 'model-a', hasKey: true, keyHint: '••••1234', workspace: '', defaultWorkspace: '/work', hideToolCalls: false, openComputerUse: false };
const store = () => reactive({ desktop: true, notify: vi.fn(), account: null, openLogin: vi.fn() }) as unknown as ControlStore;
let wrapper: VueWrapper | undefined;
beforeEach(() => { vi.resetAllMocks(); vi.mocked(listen).mockResolvedValue(() => {}); ipc.mockImplementation(async cmd => ['get_ai_config', 'refresh_ai_providers', 'save_ai_provider'].includes(cmd) ? view as never : null as never); });
afterEach(() => { wrapper?.unmount(); wrapper = undefined; document.body.innerHTML = ''; });
describe('independent provider configuration', () => {
  it('opens a native provider window from the settings list without mounting a modal', async () => {
    wrapper = mount(AiConfigCard, { props: { store: store() }, attachTo: document.body }); await flushPromises();
    await wrapper.get('[data-testid="ai-provider-config-deepseek"]').trigger('click');
    expect(ipc).toHaveBeenCalledWith('configuration_window_open', { target: 'model', tab: null, provider: 'deepseek' });
    expect(document.querySelector('[data-testid="ai-provider-dialog"]')).toBeNull();
  });
  it('renders a selected provider directly and closes its own native window only after a successful save', async () => {
    wrapper = mount(AiConfigCard, { props: { store: store(), detached: true, initialProvider: 'deepseek' }, attachTo: document.body }); await flushPromises();
    expect(wrapper.find('[data-testid="ai-provider-config-deepseek"]').exists()).toBe(false);
    expect(wrapper.get('[data-testid="ai-provider-dialog"]').attributes('aria-modal')).toBeUndefined();
    const key = wrapper.get('[data-testid="ai-api-key"] input'); await key.setValue('sk-test-new-secret');
    ipc.mockRejectedValueOnce(new Error('save failed'));
    await wrapper.get('[data-testid="ai-save"]').trigger('click'); await flushPromises();
    expect((key.element as HTMLInputElement).value).toBe('sk-test-new-secret');
    expect(ipc).not.toHaveBeenCalledWith('configuration_window_close');
    await wrapper.get('[data-testid="ai-save"]').trigger('click'); await flushPromises();
    expect(ipc).toHaveBeenCalledWith('configuration_window_close');
  });
  it('opens the account/credit provider in a native window too', async () => {
    wrapper = mount(AiGatewayCard, { props: { store: store(), config: view }, attachTo: document.body }); await flushPromises();
    await wrapper.get('[data-testid="gateway-open"]').trigger('click');
    expect(ipc).toHaveBeenCalledWith('configuration_window_open', expect.objectContaining({ target: 'model' }));
    expect(document.querySelector('[data-testid="gateway-modal"]')).toBeNull();
  });
});
