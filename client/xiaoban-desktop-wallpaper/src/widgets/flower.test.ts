// @vitest-environment happy-dom
import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import FlowerWidget from './FlowerWidget.vue';
import { FLOWER_MAX_WATER, emptyFlower, flowerDays, flowerProgress, flowerStage, validateFlower, waterUntilNext } from './flower.ts';

describe('white flower growth', () => {
  it('maps waterings to the six stages and the remaining count', () => {
    expect([0, 2, 3, 6, 7, 11, 12, 17, 18, 24, 25, 99].map(flowerStage)).toEqual([0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
    expect([0, 2, 3, 11, 18, 24, 25].map(waterUntilNext)).toEqual([3, 1, 4, 1, 7, 1, 0]);
    expect(flowerProgress(0)).toBe(0);
    expect(flowerProgress(FLOWER_MAX_WATER)).toBe(1);
    expect(flowerProgress(999)).toBe(1);
  });

  it('counts the days it has been growing from the planting day', () => {
    expect(flowerDays(emptyFlower(new Date(2026, 9, 19)).since, new Date(2026, 9, 19))).toBe(1);
    expect(flowerDays('2026-10-01', new Date(2026, 9, 19))).toBe(19);
    expect(flowerDays('not-a-date', new Date(2026, 9, 19))).toBe(1);
  });

  it('rejects impossible records instead of drawing them', () => {
    expect(() => validateFlower({ water: 3, since: '2026-10-19' })).not.toThrow();
    expect(() => validateFlower({ water: -1, since: '2026-10-19' })).toThrow();
    expect(() => validateFlower({ water: 2.5, since: '2026-10-19' })).toThrow();
    expect(() => validateFlower({ water: 1, since: '2026-02-30' })).toThrow();
    expect(() => validateFlower({ water: 1, since: '1800-01-01' })).toThrow();
  });

  it('waters by tapping, saves the whole record and paints the next stage', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const view = mount(FlowerWidget, { props: { flower: { water: 2, since: '2026-10-01' }, save } });
    expect(view.text()).toContain('再浇 1 次');
    await view.get('.flower-water').trigger('click');
    expect(save).toHaveBeenLastCalledWith({ flower: { water: 3, since: '2026-10-01' } });
    expect(view.text()).toContain('舒展');
    expect(view.findAll('.flower-art img')[1].classes()).toContain('on');
    await view.get('.flower-water').trigger('click');
    expect(save).toHaveBeenLastCalledWith({ flower: { water: 4, since: '2026-10-01' } });
    view.unmount();
  });

  it('keeps the last saved record and shows the error when saving fails', async () => {
    const save = vi.fn().mockRejectedValueOnce(new Error('磁盘已满')).mockResolvedValue(undefined);
    const view = mount(FlowerWidget, { props: { flower: { water: 5, since: '2026-10-01' }, save } });
    await view.get('.flower-water').trigger('click');
    expect(view.get('[role="alert"]').text()).toContain('磁盘已满');
    await view.get('[role="alert"]').trigger('click');
    await view.get('.flower-water').trigger('click');
    expect(save).toHaveBeenCalledTimes(2);
    view.unmount();
  });

  it('does not water from the library preview', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const view = mount(FlowerWidget, { props: { flower: emptyFlower(), preview: true, save } });
    expect(view.get('.flower-water').attributes('disabled')).toBeDefined();
    await view.get('.flower-water').trigger('click');
    expect(save).not.toHaveBeenCalled();
    view.unmount();
  });
});
