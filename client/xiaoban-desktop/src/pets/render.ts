import {faceStyleOf,skinOf,type Skin} from '../companion/skins.ts';
import {robotMarkup,type Frame} from '../companion/render.ts';
import {stillFrame} from '../companion/expressions.ts';
import type {PetDefinition,Shape,Part} from './model.ts';
import { imageSource } from './images.ts';
const escape=(v:string)=>v.replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&apos;'}[c]!));
const n=(value:number)=>Number.isFinite(value)?value.toFixed(2):'0';
function shape(s:Shape){
 if(s.type==='image')return `<image data-pet-image="true" href="${escape(imageSource(s.src))}" x="${s.x}" y="${s.y}" width="${s.width}" height="${s.height}" opacity="${s.opacity??1}" preserveAspectRatio="none"/>`;
 const attrs=`fill="${escape(s.fill??'none')}" stroke="${escape(s.stroke??'none')}" stroke-width="${s.strokeWidth??1}" opacity="${s.opacity??1}" stroke-linecap="round" stroke-linejoin="round"`;
 if(s.type==='path')return `<path d="${escape(s.d??'')}" ${attrs}/>`;
 if(s.type==='ellipse')return `<ellipse cx="${s.x}" cy="${s.y}" rx="${s.rx}" ry="${s.ry}" ${attrs}/>`;
 return `<rect x="${s.x}" y="${s.y}" width="${s.width}" height="${s.height}" rx="${s.rx??0}" ${attrs}/>`;
}
export function partTransform(part:Part,frame:Frame){
 const angle=part.motion==='none'?0:part.motion==='left'?frame.face.earL:part.motion==='right'?-frame.face.earR:Math.sin(frame.t*1.4)*2+(frame.face.earL-frame.face.earR)*.16;
 return `rotate(${n(angle)} ${part.pivot[0]} ${part.pivot[1]})`;
}
/** Keep the existing porcelain pot, face and material. Only replace its plant slot. */
export function petSkin(pet:PetDefinition,frame:Frame):Skin{
 if(pet.builtinPlant)return skinOf(pet.builtinPlant);
 const pot=skinOf('pot');
 return {...pot,earOuter:'',earInner:'',earHighlight:'',haloAt:[160,4,0,0],art:{...pot.art!,front:`<g data-part="plant-slot" transform="translate(0 -100)">${pet.parts.map(p=>`<g data-pet-part="${escape(p.id)}" transform="${partTransform(p,frame)}">${p.shapes.map(shape).join('')}</g>`).join('')}</g>`}};
}
export const petPalette=(_pet:PetDefinition)=>faceStyleOf(skinOf('pot')).palette;
export const petMarkup=(pet:PetDefinition,frame:Frame,id:string)=>{
 const markup=robotMarkup(frame,id,undefined,undefined,petSkin(pet,frame));
 // Tall flowers extend above the original plant slot. Keep them inside both the preview and native window.
 return pet.parts.some(p=>p.shapes.some(s=>s.type==='image')) ? markup.replace('viewBox="0 0 320 280"','viewBox="0 -72 320 352"') : markup;
};
const images=new WeakMap<PetDefinition,string>();
export function petImage(pet:PetDefinition){let image=images.get(pet);if(!image){image=`data:image/svg+xml;charset=utf-8,${encodeURIComponent(petMarkup(pet,stillFrame('happy'),'preview'))}`;images.set(pet,image);}return image;}

