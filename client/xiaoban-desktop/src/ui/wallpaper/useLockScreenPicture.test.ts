// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { defineComponent } from 'vue';
import { expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { useLockScreenPicture } from './useLockScreenPicture.ts';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));

it('applies the confirmed frame and keeps it available if Windows needs manual setup', async () => {
  const native = vi.mocked(invoke);
  native.mockImplementation(async command => {
    if (command === 'lock_screen_picture_supported') return true as never;
    if (command === 'lock_screen_apply_picture') return { status: 'manual', id: 'picture' } as never;
    if (command === 'lock_screen_save_picture') return true as never;
    throw new Error(command);
  });
  const notify = vi.fn();
  let controls!: ReturnType<typeof useLockScreenPicture>;
  const wrapper = mount(defineComponent({ setup() {
    controls = useLockScreenPicture({ desktop: true, notify } as unknown as ControlStore);
    return () => null;
  } }));
  try {
    await flushPromises();
    const frame = 'data:image/jpeg;base64,frame';
    await controls.show(async () => frame);
    expect(controls.picture.frame).toBe(frame);
    expect(native).not.toHaveBeenCalledWith('lock_screen_apply_picture', expect.anything());
    await controls.apply();
    expect(native).toHaveBeenCalledWith('lock_screen_apply_picture', { jpeg: frame });
    expect(controls.picture.manualId).toBe('picture');
    expect(notify).not.toHaveBeenCalled();
    await controls.save();
    expect(native).toHaveBeenLastCalledWith('lock_screen_save_picture', { id: 'picture' });
    expect(controls.picture.frame).toBe('');
    expect(notify).toHaveBeenCalledWith(expect.stringContaining('系统锁屏设置'), 'info');
  } finally {
    wrapper.unmount();
    native.mockReset();
  }
});
