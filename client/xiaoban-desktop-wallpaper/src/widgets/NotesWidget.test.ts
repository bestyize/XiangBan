// @vitest-environment happy-dom
import { mount } from '@vue/test-utils';
import { describe, it, expect, vi } from 'vitest';
import NotesWidget from './NotesWidget.vue';
import CountdownEditor from './CountdownEditor.vue';
import { emptyNote, emptyCountdown } from './content.ts';
describe('note editing',()=>{
  it('preserves unsaved edits during config events and submits the full note',async()=>{
    const save=vi.fn().mockResolvedValue(undefined),view=mount(NotesWidget,{props:{note:emptyNote(),save}});
    await view.get('textarea').setValue('别忘了喝水');await view.setProps({note:{...emptyNote(),title:'其他窗口更新'}});
    expect((view.get('textarea').element as HTMLTextAreaElement).value).toBe('别忘了喝水');
    await view.get('footer button').trigger('click');expect(save).toHaveBeenCalledWith({note:{...emptyNote(),text:'别忘了喝水'}});expect(view.text()).not.toContain('未保存');view.unmount();
  });
  it('adds, completes and deletes a task without dragging the input',async()=>{
    const save=vi.fn().mockResolvedValue(undefined),view=mount(NotesWidget,{props:{note:{...emptyNote(),mode:'todo'},save}});
    await view.get('[aria-label="新增待办"]').setValue('整理桌面');await view.get('form').trigger('submit');expect(view.get('.todo-item').text()).toContain('整理桌面');
    await view.get('[type="checkbox"]').trigger('change');expect(save.mock.lastCall?.[0].note.items[0].done).toBe(true);
    await view.get('[aria-label="删除：整理桌面"]').trigger('click');expect(save.mock.lastCall?.[0].note.items).toEqual([]);view.unmount();
  });
  it('keeps failed saves visible and retries the draft',async()=>{
    const save=vi.fn().mockRejectedValueOnce(new Error('磁盘已满')).mockResolvedValue(undefined),view=mount(NotesWidget,{props:{note:emptyNote(),save}});
    await view.get('textarea').setValue('重要记录');await view.get('footer button').trigger('click');expect(view.get('[role="alert"]').text()).toContain('磁盘已满');expect(view.text()).toContain('未保存');await view.get('footer button').trigger('click');expect(save).toHaveBeenCalledTimes(2);expect(view.find('[role="alert"]').exists()).toBe(false);view.unmount();
  });
});
describe('anniversary settings',()=>{
  it('clears annual repetition when switching to elapsed-day mode',async()=>{
    const save=vi.fn().mockResolvedValue(undefined),view=mount(CountdownEditor,{props:{data:{...emptyCountdown(),repeat:'yearly'},save}});
    await view.get('[aria-label="计算方式"]').setValue('elapsed');await view.get('form').trigger('submit');expect(save.mock.lastCall?.[0].countdown).toMatchObject({mode:'elapsed',repeat:'none'});view.unmount();
  });
});
