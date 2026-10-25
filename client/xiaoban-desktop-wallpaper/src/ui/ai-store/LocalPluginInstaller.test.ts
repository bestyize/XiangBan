// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { invoke, listen } from '../../bridge.ts';
import LocalPluginInstaller from './LocalPluginInstaller.vue';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn(), listen: vi.fn() }));
vi.mock('naive-ui', async () => {
  const { defineComponent } = await import('vue');
  return { NModal: defineComponent({ props: ['show'], template: '<section v-if="show"><slot /></section>' }) };
});
let wrapper: VueWrapper;
let drop: (payload: { paths: string[] }) => void;
const off = vi.fn();
const review = { name: 'repository', version: '1.0', kind: 'connector', description: '代码工具', manifestSha256: 'a'.repeat(64), agents: [], skills: ['review'], hasMcp: true, warnings: ['请核实来源'] };
beforeEach(() => {
  vi.mocked(invoke).mockReset().mockImplementation(async command => command === 'ai_plugin_inspect_file' ? review : null);
  vi.mocked(listen).mockReset().mockImplementation(async (_event, callback) => { drop = callback as typeof drop; return off; });
  off.mockReset();
});
afterEach(() => wrapper?.unmount());
async function open() { wrapper = mount(LocalPluginInstaller, { props: { desktop: true } }); await wrapper.get('button').trigger('click'); await flushPromises(); }
async function inspect(path = 'C:\\plugins\\repository.zip') { await wrapper.get('[aria-label="插件包完整路径"]').setValue(path); await wrapper.get('form').trigger('submit'); await flushPromises(); }
it('reviews before installation and locks the installed package to its digest', async () => {
  await open(); await inspect();
  expect(wrapper.text()).toContain('repository · v1.0'); expect(wrapper.text()).toContain('技能 1');
  expect(wrapper.text()).toContain('请核实来源'); expect(wrapper.text()).toContain('安装后需要你确认并配置才会连接');
  expect(invoke).not.toHaveBeenCalledWith('ai_plugin_install_file', expect.anything());
  const install = wrapper.findAll('button').find(b => b.text() === '确认安装')!;
  expect(install.attributes('disabled')).toBeDefined();
  await wrapper.get('input[type=checkbox]').setValue(true); await install.trigger('click'); await flushPromises();
  expect(invoke).toHaveBeenCalledWith('ai_plugin_install_file', { path: 'C:\\plugins\\repository.zip', sha256: review.manifestSha256, confirmed: true });
  expect(wrapper.emitted('installed')).toHaveLength(1);
});
it('opens the system file dialog on open and reviews what was chosen; cancelling reviews nothing', async () => {
  await open();
  expect(invoke).toHaveBeenCalledWith('ai_plugin_pick_file');
  expect(invoke).not.toHaveBeenCalledWith('ai_plugin_inspect_file', expect.anything());
  expect(wrapper.find('input[type=file]').exists()).toBe(false);
  vi.mocked(invoke).mockImplementation(async command => command === 'ai_plugin_pick_file' ? 'D:/downloads/repository.zip' : command === 'ai_plugin_inspect_file' ? review : null);
  await wrapper.findAll('button').find(b => b.text() === '选择 ZIP 文件')!.trigger('click'); await flushPromises();
  expect(invoke).toHaveBeenCalledWith('ai_plugin_inspect_file', { path: 'D:/downloads/repository.zip' });
  expect(wrapper.text()).toContain('repository · v1.0');
});
it('only reviews ZIP files with a full path', async () => {
  await open();
  await inspect('C:\fakepath\repository.txt');
  expect(invoke).not.toHaveBeenCalledWith('ai_plugin_inspect_file', expect.anything());
  expect(wrapper.text()).toContain('完整路径');
});
it('accepts native dropped ZIP paths and removes the listener when destroyed', async () => {
  await open(); drop({ paths: ['/Users/me/repository.zip'] }); await flushPromises();
  expect(invoke).toHaveBeenCalledWith('ai_plugin_inspect_file', { path: '/Users/me/repository.zip' });
  wrapper.unmount(); expect(off).toHaveBeenCalled();
});
it('invalidates a review when the path changes and surfaces install errors', async () => {
  await open(); await inspect();
  await wrapper.get('[aria-label="插件包完整路径"]').setValue('C:\\other.zip'); expect(wrapper.find('input[type=checkbox]').exists()).toBe(false);
  await inspect(); await wrapper.get('input[type=checkbox]').setValue(true);
  vi.mocked(invoke).mockRejectedValueOnce('插件包与审查时不一致');
  await wrapper.findAll('button').find(b => b.text() === '确认安装')!.trigger('click'); await flushPromises();
  expect(wrapper.text()).toContain('插件包与审查时不一致'); expect(wrapper.emitted('installed')).toBeUndefined();
});
it('requires a review digest and reports inspection errors', async () => {
  await open(); vi.mocked(invoke).mockRejectedValueOnce('无效 ZIP'); await inspect(); expect(wrapper.text()).toContain('无效 ZIP');
  vi.mocked(invoke).mockResolvedValueOnce({ ...review, manifestSha256: '' }); await inspect(); expect(wrapper.text()).toContain('缺少清单摘要');
  expect(wrapper.find('input[type=checkbox]').exists()).toBe(false);
});
