import assert from 'node:assert/strict'
import test from 'node:test'
import { collectionProjects, filterProjects, findProject, openSourceCatalog as catalog } from '../src/lib/openSource.ts'

test('a project can be discovered in related categories without becoming a duplicate', () => {
  const office = filterProjects(catalog.projects, 'office')
  const ai = filterProjects(catalog.projects, 'ai')
  assert.ok(office.some(item => item.full_name === 'AppFlowy-IO/AppFlowy'))
  assert.ok(ai.some(item => item.full_name === 'AppFlowy-IO/AppFlowy'))
  assert.equal(new Set(ai.map(item => item.full_name)).size, ai.length)
  assert.deepEqual(filterProjects(catalog.projects, 'unknown'), [])
})

test('Chinese search, repository search and tags combine with the selected category', () => {
  assert.equal(filterProjects(catalog.projects, 'media', '字幕')[0]?.name, 'yt-dlp')
  assert.equal(filterProjects(catalog.projects, 'skills', 'MCP', 'MCP')[0]?.name, 'MCP Servers')
  assert.equal(filterProjects(catalog.projects, 'design', 'appflowy').length, 0)
  assert.equal(filterProjects(catalog.projects, 'office', ' APPFLOWY-IO/APPFLOWY ')[0]?.name, 'AppFlowy')
  assert.equal(filterProjects(catalog.projects, 'recommend', 'does-not-exist').length, 0)
})

test('recommendations require editorial selection and collection references share the same project', () => {
  const excluded = { ...catalog.projects[0]!, featured: false }
  assert.deepEqual(filterProjects([excluded], 'recommend'), [])
  assert.deepEqual(filterProjects([excluded], 'all', excluded.name), [excluded], 'global search also finds projects outside recommendations')
  assert.equal(findProject('EXCALIDRAW/EXCALIDRAW'), catalog.projects[0])
  const collection = catalog.collections[0]!
  assert.equal(collectionProjects(collection)[0], findProject(collection.projects[0]!))
  assert.deepEqual(collectionProjects({ ...collection, projects: ['missing/repo'] }), [])
})

// The API, browser preview and exported covers must resolve the same template.
import cases from '../../editorial/open-source-cover-cases.json' with { type: 'json' }
import { coverHash, openSourceCoverRules, resolveOpenSourceCover } from '../src/lib/openSourceCovers.ts'
test('curated cover rules share backend protocol cases and stable repository identity', () => {
  assert.equal(openSourceCoverRules.scope, 'open-source')
  for (const entry of cases) {
    const result = resolveOpenSourceCover({ ...catalog.projects[0]!, cover_style: undefined, cover_layout: undefined, cover_accent: undefined, cover_variant: undefined, cover_lines: [], ...entry.input })
    for (const [key, value] of Object.entries(entry.expected)) assert.deepEqual(result[key as keyof typeof result], value)
  }
  assert.equal(coverHash('Example/Tool'), coverHash('EXAMPLE/TOOL'))
  const original = { ...catalog.projects[0]!, order: 1 }, reordered = { ...original, order: 100 }
  assert.deepEqual(resolveOpenSourceCover(original), { ...resolveOpenSourceCover(reordered), order: 1 })
})
