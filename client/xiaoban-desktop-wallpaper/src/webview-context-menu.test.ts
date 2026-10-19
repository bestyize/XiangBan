// @vitest-environment happy-dom
import {afterEach,describe,it,expect,vi} from 'vitest';
import source from '../src-tauri/src/webview_context_menu.js?raw';
function frame(){const iframe=document.createElement('iframe');document.body.append(iframe);const win=iframe.contentWindow!;new Function('window',source)(win);return {win,document:iframe.contentDocument!};}
afterEach(()=>document.body.replaceChildren());
describe('native WebView context menus',()=>{
 it('prevents the browser menu on background and editable text',()=>{const f=frame();for(const tag of ['div','input','textarea']){const node=f.document.createElement(tag);f.document.body.append(node);const event=new MouseEvent('contextmenu',{bubbles:true,cancelable:true});node.dispatchEvent(event);expect(event.defaultPrevented).toBe(true);}});
 it('still delivers right-click events to application menus',()=>{const f=frame(),node=f.document.createElement('button'),custom=vi.fn();f.document.body.append(node);node.addEventListener('contextmenu',custom);node.dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,cancelable:true}));expect(custom).toHaveBeenCalledOnce();});
 it('does not block other pointer actions',()=>{const f=frame(),node=f.document.createElement('button'),click=vi.fn();f.document.body.append(node);node.addEventListener('click',click);const event=new MouseEvent('click',{bubbles:true,cancelable:true});node.dispatchEvent(event);expect(click).toHaveBeenCalledOnce();expect(event.defaultPrevented).toBe(false);});
});
