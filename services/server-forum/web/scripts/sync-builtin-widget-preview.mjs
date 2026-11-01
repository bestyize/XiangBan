// Synchronize the desktop's visual renderer. The web adapter has no native bridge.
import { copyFile, mkdir, readFile, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
const web = new URL('../', import.meta.url)
const source = new URL('../../../../client/xiaoban-desktop/src/widgets/', import.meta.url)
const dest = new URL('src/components/creation/widget-preview/', web)
await mkdir(new URL('assets/', dest), { recursive: true })
for (const name of ['model.ts','content.ts','calendar.ts','holidays.json','ItemForm.vue','resize.ts','custom-drag.ts','custom-appearance.ts','surface.css','assets/design-reference.png','assets/woodfish-body-mask.svg','assets/woodfish-mallet-mask.svg']) {
  if (/\.(?:ts|vue)$/.test(name)) { const contents = await readFile(new URL(name, source), 'utf8'); await writeFile(new URL(name, dest), contents.replace(/(from\s+['\"]\.[^'\"]+)\.ts(['\"])/g, '$1$2')) }
  else await copyFile(new URL(name, source), new URL(name, dest))
}
let surface = await readFile(new URL('WidgetSurface.vue', source), 'utf8')
surface = surface.replace("from 'lucide-vue-next'", "from './icons'").replace("from '../bridge.ts'", "from './bridge'").replace("import './surface.css';", '')
surface = surface.replace(/(from\s+['\"]\.[^'\"]+)\.ts(['\"])/g, '$1$2')
surface += '\n<style scoped src="./surface.css"></style>\n'
await writeFile(new URL('WidgetSurface.vue', dest), surface)
await copyFile(new URL('../../../../client/xiaoban-desktop/public/widgets/pelican-bike.json', import.meta.url), new URL('pelican-bike.json', dest))
console.log('Desktop widget preview synchronized:', fileURLToPath(dest))
const model = await readFile(new URL('model.ts', source), 'utf8')
const block = model.match(/export const widgetKinds = \[([\s\S]*?)\];/)
if (!block) throw new Error('Desktop widgetKinds not found')
const kinds = [...block[1].matchAll(/kind:'([^']+)'\s+as const,title:'([^']+)',label:'[^']+',description:'([^']+)'/g)]
if (kinds.length !== 8) throw new Error(`Unexpected desktop widgetKinds count: ${kinds.length}`)
const custom = JSON.parse(await readFile(new URL('pelican-bike.json', dest), 'utf8'))
const rows = kinds.map(([,id,name,description]) => ({id,name,description,kind:'widget'}))
rows.push({ id:custom.id, name:custom.name, description:custom.description, kind:'widget' })
await writeFile(new URL('src/lib/builtinWidgets.ts', web), `/** Generated from desktop widgetKinds and bundled custom widget. */\nexport const BUILTIN_WIDGETS: { id: string; name: string; description: string; kind: 'widget' }[] = ${JSON.stringify(rows,null,2)}\n`)
