// @vitest-environment happy-dom
import { mount } from '@vue/test-utils';
import { reactive, nextTick } from 'vue';
import { afterEach, describe, expect, it, vi } from 'vitest';
import AssistantApp from './AssistantApp.vue';
import type { AssistantStore } from './assistant-store.ts';
import type { AsrStore } from '../ui/asr/store.ts';

vi.mock('../bridge.ts', () => ({ invoke: vi.fn(async () => {}) }));
vi.mock('./Orb.vue', () => ({ default: { template: '<span />' } }));
vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });

const wrappers: ReturnType<typeof mount>[] = [];
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); });
function setup() {
  const store = reactive({ open: true, ball: false, phase: 'answering', hint: '', hintTab: '', question: '今天如何放松', answer: '可以先休息片刻。', working: '', failure: '', approval: null as null | { id: string; arguments: {}; tool: string }, ask: null, input: '草稿', status: '正在回答…', hide: vi.fn(), submit: vi.fn(), interrupt: vi.fn(), openChat: vi.fn() });
  const wrapper=mount(AssistantApp,{props:{store:store as unknown as AssistantStore,asr:{level:0} as AsrStore},global:{stubs:{MarkdownView:true}}});
  wrappers.push(wrapper);
  return {store,wrapper};
}

describe('assistant response card', () => {
  it('closes only the upper card while leaving the capsule, draft and running reply untouched', async () => {
    const {store,wrapper}=setup();
    await wrapper.get('[data-testid="assistant-card-close"]').trigger('click');
    expect(wrapper.find('[data-testid="assistant-card"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="assistant"]').exists()).toBe(true);
    expect(wrapper.get<HTMLInputElement>('[data-testid="assistant-input"]').element.value).toBe('草稿');
    expect(store.hide).not.toHaveBeenCalled();
    expect(store.interrupt).not.toHaveBeenCalled();
    store.answer += '正在继续回答';await nextTick();
    expect(wrapper.find('[data-testid="assistant-card"]').exists()).toBe(false);
    store.question='新的问题';await nextTick();
    expect(wrapper.find('[data-testid="assistant-card"]').exists()).toBe(true);
  });

  it('reveals a fresh approval even after the card was dismissed', async () => {
    const {store,wrapper}=setup();
    await wrapper.get('[data-testid="assistant-card-close"]').trigger('click');
    store.approval={id:'approval1',arguments:{},tool:'shell'};await nextTick();
    expect(wrapper.find('[data-testid="assistant-approval"]').exists()).toBe(true);
  });
});
