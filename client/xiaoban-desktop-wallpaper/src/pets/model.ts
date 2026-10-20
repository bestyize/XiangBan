import official from '../../public/pets/catalog.json';
import { imageSource, validateImage } from './images.ts';
const builtinImages = import.meta.glob<string>('./assets/*.webp', { eager: true, query: '?inline', import: 'default' });
import { EXPRESSION_NAMES, type ExpressionName } from '../companion/expressions.ts';

export interface Shape {
  type: 'path' | 'ellipse' | 'rect' | 'image';
  src?: string;
  d?: string; x?: number; y?: number; width?: number; height?: number; rx?: number; ry?: number;
  fill?: string; stroke?: string; strokeWidth?: number; opacity?: number;
}
export interface Part { id: string; name: string; pivot: [number, number]; motion: 'none' | 'sway' | 'left' | 'right'; shapes: Shape[] }
export interface PetDefinition {
  id: string; name: string; description: string; kind: string;
  builtinPlant?: 'pot' | 'pot-white-flower';
  parts: Part[];
}
export interface PetTheme { schemaVersion: 1 | 2; id: string; name: string; description: string; color: string; pets: PetDefinition[] }
export interface PetInstance { id: string; themeId: string; petId: string; name: string; size: number; x: number; y: number; layer: 'desktop' | 'topmost'; enabled: boolean; expression: ExpressionName; links: boolean; monitor?: string }
export interface PetLayout { version: 1; instances: PetInstance[] }
export interface PetCatalog { themes: (PetTheme & { source: 'builtin' | 'custom' })[]; layout: PetLayout; errors: string[] }
export const OFFICIAL_THEMES = (official.themes as PetTheme[]).map(theme => ({...theme, pets: theme.pets.map(pet => ({...pet, parts: pet.parts.map(part => ({...part, shapes: part.shapes.map(shape => shape.type === 'image' ? {...shape, src: imageSource(builtinImages['./'+shape.src])} : shape)}))}))}));
export const definitionOf = (themes: PetTheme[], instance: PetInstance) => themes.find(t => t.id === instance.themeId)?.pets.find(p => p.id === instance.petId);
export const validId = (id: unknown): id is string => typeof id === 'string' && /^[a-z0-9][a-z0-9_-]{0,47}$/.test(id);
const color = (value: unknown) => typeof value === 'string' && /^(#[\da-f]{6}|none)$/i.test(value);
const number = (value: unknown, min: number, max: number): value is number => typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
const text = (value: unknown, max: number) => typeof value === 'string' && value.trim().length > 0 && value.length <= max;
export function validateTheme(value: unknown): PetTheme {
  const theme=value as PetTheme;
  if(!theme || ![1,2].includes(theme.schemaVersion) || !validId(theme.id) || !text(theme.name,24) || !text(theme.description,160) || !color(theme.color) || !Array.isArray(theme.pets) || !theme.pets.length || theme.pets.length>40)throw Error('宠物主题格式不正确');
  const bytes = new TextEncoder().encode(JSON.stringify(theme)).length;
  if(bytes > (theme.schemaVersion===1 ? 512*1024 : 24*1024*1024))throw Error('宠物主题文件过大');
  const ids=new Set<string>();
  let imageBytes=0,imageCount=0;
  const shapes=(items:Shape[])=>{
    if(!Array.isArray(items)||!items.length||items.length>100)throw Error('宠物图形数量无效');
    for(const s of items){
      if(s?.type==='image'){
        if(theme.schemaVersion!==2)throw Error('包含图片部件需要 schemaVersion=2');
        const size=validateImage(s.src);
        imageBytes+=size;imageCount++;
        if(imageCount>60||imageBytes>16*1024*1024)throw Error('宠物素材总量超过限制');
        if(!number(s.x,0,320)||!number(s.y,-64,320)||!number(s.width,1,320)||!number(s.height,1,320)||(s.opacity!=null&&!number(s.opacity,0,1)))throw Error('宠物图片尺寸无效');
        continue;
      }
      if(!s||!['path','ellipse','rect'].includes(s.type)||!color(s.fill)||(s.stroke!=null&&!color(s.stroke))||(s.strokeWidth!=null&&!number(s.strokeWidth,0,12))||(s.opacity!=null&&!number(s.opacity,0,1)))throw Error('宠物图形样式无效');
      if(s.type==='path'){if(typeof s.d!=='string'||!s.d.trim()||s.d.length>16000||!/^[MmZzLlHhVvCcSsQqTtAaEe\d\s.,+\-]+$/.test(s.d))throw Error('宠物路径无效');}
      else if(!number(s.x,0,320)||!number(s.y,0,320)||(s.type==='ellipse'?(!number(s.rx,0,160)||!number(s.ry,0,160)):(!number(s.width,1,320)||!number(s.height,1,320)||(s.rx!=null&&!number(s.rx,0,160)))))throw Error('宠物图形尺寸无效');
    }
  };
  for(const pet of theme.pets){
    if(!pet||!validId(pet.id)||ids.has(pet.id)||!text(pet.name,24)||!text(pet.description,160)||!validId(pet.kind))throw Error('宠物名称或标识无效');ids.add(pet.id);
    if(pet.builtinPlant!==undefined&&!['pot','pot-white-flower'].includes(pet.builtinPlant))throw Error('内置宠物形象无效');
    if(!Array.isArray(pet.parts)||pet.parts.length>16||(!pet.builtinPlant&&!pet.parts.length)||(pet.builtinPlant&&pet.parts.length>0))throw Error('宠物部件数量无效');
    const parts=new Set<string>();
    for(const part of pet.parts){if(!validId(part.id)||parts.has(part.id)||!text(part.name,24)||!['none','sway','left','right'].includes(part.motion)||!Array.isArray(part.pivot)||part.pivot.length!==2||!part.pivot.every(n=>number(n,0,320)))throw Error('宠物部件无效');parts.add(part.id);shapes(part.shapes);}
  }
  return theme;
}
export function validateLayout(value: unknown): PetLayout {
  const layout=value as PetLayout,ids=new Set<string>();
  if(!layout||layout.version!==1||!Array.isArray(layout.instances))throw Error('桌面宠物布局格式不正确');
  for(const p of layout.instances){if(!p||!validId(p.id)||ids.has(p.id)||!validId(p.themeId)||!validId(p.petId)||!text(p.name,40)||!number(p.size,80,420)||!number(p.x,0,1)||!number(p.y,0,1)||!['desktop','topmost'].includes(p.layer)||typeof p.enabled!=='boolean'||typeof p.links!=='boolean'||!EXPRESSION_NAMES.includes(p.expression)||(p.monitor!==undefined&&!text(p.monitor,512)))throw Error('桌面宠物实例无效');ids.add(p.id);}
  return layout;
}

