import {createApp} from 'vue';
import {applyUiTokens} from '../design-tokens.ts';
import '../control.css';
import DesignWindow from './DesignWindow.vue';
export function mountDesign(){
 applyUiTokens();document.body.style.margin='0';
 const app=createApp(DesignWindow);app.mount(document.querySelector('#app')!);
 window.addEventListener('beforeunload',()=>app.unmount(),{once:true});
}
