import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import { applyFlow, closeApply, confirmApply, scheduleText, schemeFlow, schemeProgress, startApply, startScheme, stopScheme, type PackPreview } from './apply.ts';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn(), listen: vi.fn() }));

const preview: PackPreview = {
  token: 't1', packId: 7, kind: 'reminder', title: '喝水提醒', version: '1.0.0', trustLevel: 0, canApply: true,
  size: 900, sha256: 'a'.repeat(64), items: [{ title: 'custom', detail: '喝水', schedule: { repeat: 'daily', time: '10:00' } }], startsDisabled: false,
};

describe('community apply flow', () => {
  beforeEach(() => { vi.mocked(invoke).mockReset(); closeApply(); });

  it('checks first and only applies after the user confirms', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(preview);
    await startApply({ packId: 7, version: 2, sha256: 'b'.repeat(64) });
    expect(invoke).toHaveBeenCalledWith('community_prepare', { packId: 7, version: 2, sha256: 'b'.repeat(64) });
    expect(applyFlow.state.phase).toBe('confirm');
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === 'community_apply')).toBe(false);

    vi.mocked(invoke).mockResolvedValueOnce({ message: '已添加 1 条提醒' });
    expect(await confirmApply()).toBe('已添加 1 条提醒');
    expect(invoke).toHaveBeenLastCalledWith('community_apply', { token: 't1' });
    expect(applyFlow.state).toEqual({ phase: 'done', message: '已添加 1 条提醒' });
  });

  it('shows a failed check and never reaches apply', async () => {
    vi.mocked(invoke).mockRejectedValueOnce('文件校验失败（SHA-256 不一致）');
    await startApply({ packId: 7 });
    expect(applyFlow.state).toEqual({ phase: 'error', message: '文件校验失败（SHA-256 不一致）' });
    expect(await confirmApply()).toBeNull();
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it('turns an apply failure into a message instead of throwing', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(preview).mockRejectedValueOnce(new Error('最多保存 50 条提醒'));
    await startApply({ packId: 7 });
    expect(await confirmApply()).toBeNull();
    expect(applyFlow.state).toEqual({ phase: 'error', message: '最多保存 50 条提醒' });
  });

  it('hands a task template to the console instead of applying anything', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ ...preview, kind: 'task', items: [{ title: '整理', detail: 'p' }] })
      .mockResolvedValueOnce({ status: 'task', message: '已放进新对话的输入框，确认后再发送', task: { prompt: '请整理下载文件夹' } });
    await startApply({ packId: 8 });
    const task = vi.fn();
    expect(await confirmApply({ task, goal: vi.fn() })).toContain('输入框');
    expect(task).toHaveBeenCalledWith('请整理下载文件夹');
  });

  it('hands a goal template to the goal form', async () => {
    const goal = { outcome: '完成开题报告', verification: '', constraints: '', boundaries: '', stopWhen: '' };
    vi.mocked(invoke).mockResolvedValueOnce({ ...preview, kind: 'goal' }).mockResolvedValueOnce({ status: 'goal', message: '已填入长期目标表单，确认后再创建', goal });
    await startApply({ packId: 9 });
    const handler = vi.fn();
    await confirmApply({ task: vi.fn(), goal: handler });
    expect(handler).toHaveBeenCalledWith(goal);
  });

  it('replicates a scheme one item at a time and skips what the desktop cannot apply', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ ...preview, packId: 1, kind: 'effect' });
    const count = startScheme('我的桌面', [{ pack_id: 1, kind: 'effect' }, { pack_id: 2, kind: 'wallpaper' }, { pack_id: 3, kind: 'reminder', version: 2 }]);
    expect(count).toBe(2);
    expect(schemeFlow.skipped).toBe(1);
    await vi.waitFor(() => expect(applyFlow.state.phase).toBe('confirm'));
    expect(schemeProgress()).toContain('第 1 / 2 项');
    // 每一项都要确认：到下一项之前，第二项还没有开始下载。
    expect(vi.mocked(invoke).mock.calls.filter(([c]) => c === 'community_prepare')).toHaveLength(1);

    vi.mocked(invoke).mockResolvedValueOnce({ ...preview, packId: 3 });
    closeApply();
    await vi.waitFor(() => expect(applyFlow.state.phase).toBe('confirm'));
    expect(invoke).toHaveBeenLastCalledWith('community_prepare', { packId: 3, version: 2, sha256: null });
    expect(schemeProgress()).toContain('第 2 / 2 项');

    closeApply();
    expect(applyFlow.state.phase).toBe('idle');
    expect(schemeFlow.total).toBe(0);
  });

  it('stops a scheme in the middle', async () => {
    vi.mocked(invoke).mockResolvedValue({ ...preview });
    startScheme('x', [{ pack_id: 1, kind: 'effect' }, { pack_id: 2, kind: 'effect' }]);
    await vi.waitFor(() => expect(applyFlow.state.phase).toBe('confirm'));
    stopScheme();
    expect(applyFlow.state.phase).toBe('idle');
    expect(schemeFlow.queue).toHaveLength(0);
    expect(vi.mocked(invoke).mock.calls.filter(([c]) => c === 'community_prepare')).toHaveLength(1);
  });

  it('describes schedules in plain words', () => {
    expect(scheduleText({ repeat: 'daily', time: '08:30' })).toBe('每天 08:30');
    expect(scheduleText({ repeat: 'weekdays', time: '09:00', weekdays: [1, 2, 3, 4, 5] })).toBe('每周一、二、三、四、五 09:00');
    expect(scheduleText({ intervalMinutes: 90 })).toBe('每 90 分钟');
    expect(scheduleText(undefined)).toBe('');
  });
});
