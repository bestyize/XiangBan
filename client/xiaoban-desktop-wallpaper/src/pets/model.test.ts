import {describe,it,expect} from 'vitest';
import {OFFICIAL_THEMES,validateTheme,validateLayout} from './model.ts';
import {robotMarkup} from '../companion/render.ts';
import {petMarkup} from './render.ts';
import {EXPRESSION_NAMES,stillFrame} from '../companion/expressions.ts';
describe('pet works and expression rendering',()=>{
 it('ships two plant themes whose seven works validate and support every expression',()=>{expect(OFFICIAL_THEMES.map(t=>t.name)).toEqual(['花朵心语','多肉点点']);let count=0;for(const t of OFFICIAL_THEMES){expect(validateTheme(t)).toBe(t);for(const p of t.pets){count++;for(const expression of EXPRESSION_NAMES){const svg=petMarkup(p,stillFrame(expression),'one');expect(svg).not.toContain('NaN');expect(svg).toContain('data-part="face"');expect(svg).toContain('screenClip');}}}expect(count).toBe(7);});
 it('preserves both original pot designs exactly',()=>{for(const p of OFFICIAL_THEMES[0].pets.filter(p=>p.builtinPlant))for(const e of EXPRESSION_NAMES){const f=stillFrame(e);expect(petMarkup(p,f,'same')).toBe(robotMarkup(f,'same',undefined,p.builtinPlant));}});
 it('rejects executable artwork and external fills',()=>{const theme=structuredClone(OFFICIAL_THEMES[0]);theme.pets[2].parts[0].shapes[0].fill='url(https://example.com/a)';expect(()=>validateTheme(theme)).toThrow();theme.pets[2].parts[0].shapes[0].fill='#f4e9d8';theme.pets[2].parts[0].shapes[0].d='<script>alert(1)</script>';expect(()=>validateTheme(theme)).toThrow();});
 it('keeps independent positions, sizes, moods, and duplicates across layout serialization',()=>{const one={id:'one',themeId:'flower-whispers',petId:'daisy',name:'花',size:180,x:.2,y:.3,layer:'topmost',enabled:true,expression:'idle',links:true};const layout=validateLayout({version:1,instances:[one,{...one,id:'two',size:300,expression:'sleeping'}]});const restored=validateLayout(JSON.parse(JSON.stringify(layout)));restored.instances[0].expression='happy';expect(restored.instances[1].expression).toBe('sleeping');expect(restored.instances[1].size).toBe(300);expect(()=>validateLayout({version:1,instances:[one,one]})).toThrow();});
});
