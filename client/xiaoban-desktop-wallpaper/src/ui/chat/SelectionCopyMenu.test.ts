// @vitest-environment happy-dom
import { mount,flushPromises } from '@vue/test-utils';
import { afterEach,beforeEach,it,expect,vi } from 'vitest';
import SelectionCopyMenu from './SelectionCopyMenu.vue';
let wrapper:ReturnType<typeof mount>,scope:HTMLElement;
const writeText=vi.fn(async()=>{});
beforeEach(()=>{
  document.body.innerHTML='<section id="messages"><p>开头 只复制这些 结尾</p></section><aside>其他区域</aside>';
  scope=document.getElementById('messages')!;
  Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText}});writeText.mockReset().mockResolvedValue(undefined);
  wrapper=mount(SelectionCopyMenu,{props:{scope},attachTo:document.body});
});
afterEach(()=>{wrapper.unmount();window.getSelection()?.removeAllRanges();document.body.replaceChildren();vi.restoreAllMocks();});
function select(){const range=document.createRange(),text=scope.querySelector('p')!.firstChild!;range.setStart(text,3);range.setEnd(text,8);const selection=window.getSelection()!;selection.removeAllRanges();selection.addRange(range);}
async function rightClick(target:HTMLElement=scope.querySelector('p')!){const event=new MouseEvent('contextmenu',{bubbles:true,cancelable:true,clientX:200,clientY:180});target.dispatchEvent(event);await flushPromises();return event;}
it('copies exactly the selected substring and closes the menu',async()=>{
  select();expect((await rightClick()).defaultPrevented).toBe(true);
  const button=document.querySelector<HTMLButtonElement>('[role="menuitem"]')!;expect(button.textContent).toBe('复制');
  button.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true,cancelable:true}));button.click();await flushPromises();
  expect(writeText).toHaveBeenCalledWith('只复制这些');expect(document.querySelector('[role="menu"]')).toBeNull();expect(wrapper.emitted('visibilityChange')).toBeTruthy();
});
it('does not show copy with an empty selection or outside the conversation',async()=>{
  await rightClick();expect(document.querySelector('[role="menu"]')).toBeNull();select();await rightClick(document.querySelector('aside')!);expect(document.querySelector('[role="menu"]')).toBeNull();
});
it('closes on Escape and does not pass it on to the assistant',async()=>{
  select();await rightClick();const bubbled=vi.fn();document.body.addEventListener('keydown',bubbled);
  document.body.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));await flushPromises();
  expect(document.querySelector('[role="menu"]')).toBeNull();expect(bubbled).not.toHaveBeenCalled();
});
it('shows retry when copying fails without pretending it succeeded',async()=>{
  writeText.mockRejectedValueOnce(new Error('denied'));select();await rightClick();document.querySelector<HTMLButtonElement>('[role="menuitem"]')!.click();await flushPromises();
  expect(document.querySelector('[role="menuitem"]')?.textContent).toBe('复制失败，重试');
});
it('dismisses before an outside click',async()=>{
  select();await rightClick();document.querySelector('aside')!.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));await flushPromises();expect(document.querySelector('[role="menu"]')).toBeNull();
});
