import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import { applyFlow, closeApply, confirmApply, scheduleText, startApply, type PackPreview } from './apply.ts';

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

  it('describes schedules in plain words', () => {
    expect(scheduleText({ repeat: 'daily', time: '08:30' })).toBe('每天 08:30');
    expect(scheduleText({ repeat: 'weekdays', time: '09:00', weekdays: [1, 2, 3, 4, 5] })).toBe('每周一、二、三、四、五 09:00');
    expect(scheduleText({ intervalMinutes: 90 })).toBe('每 90 分钟');
    expect(scheduleText(undefined)).toBe('');
  });
});
