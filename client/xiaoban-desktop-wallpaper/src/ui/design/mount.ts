import {createApp} from 'vue';
import '../control.css';
import DesignWindow from './DesignWindow.vue';
export function mountDesign(){
 document.body.classList.add('desktop-control');
 const app=createApp(DesignWindow);app.mount(document.querySelector('#app')!);
 window.addEventListener('beforeunload',()=>app.unmount(),{once:true});
}
