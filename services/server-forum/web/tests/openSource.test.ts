import assert from 'node:assert/strict'
import test from 'node:test'
import selection from '../../editorial/open-source-selection-20261103.json' with { type: 'json' }
import { collectionProjects, filterProjects, findProject, openSourceCatalog as catalog } from '../src/lib/openSource.ts'

test('the reviewed selection uses verified stars, recent default-branch commits and covers every category', () => {
  const reviewed = Date.parse(`${selection.reviewed_at}T23:59:59Z`)
  const cutoff = Date.parse(selection.policy.cutoff)
  const verified = new Map(selection.selected.map(item => [item.full_name.toLowerCase(), item]))
  let previousStars = Infinity
  for (const project of filterProjects(catalog.projects, 'all')) {
    const source = verified.get(project.full_name.toLowerCase())!
    assert.ok(source, project.full_name)
    assert.ok(!source.archived && !source.disabled, project.full_name)
    assert.ok(Date.parse(source.last_commit_at) >= cutoff && Date.parse(source.last_commit_at) <= reviewed, project.full_name)
    assert.ok(source.stargazers_count <= previousStars, 'Star count is the primary ordering key')
    previousStars = source.stargazers_count
  }
  assert.equal(verified.size, catalog.projects.length)
  for (const category of catalog.categories) {
    assert.ok(catalog.projects.filter(project => project.category === category.key).length >= 15, `${category.key}: at least 15 primary projects`)
    assert.ok(filterProjects(catalog.projects, category.key).length >= 15, category.key)
  }
})

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
  assert.ok(filterProjects(catalog.projects, 'skills', 'MCP', 'MCP').some(item => item.name === 'MCP Servers'))
  assert.equal(filterProjects(catalog.projects, 'design', 'appflowy').length, 0)
  assert.equal(filterProjects(catalog.projects, 'office', ' APPFLOWY-IO/APPFLOWY ')[0]?.name, 'AppFlowy')
  assert.equal(filterProjects(catalog.projects, 'recommend', 'does-not-exist').length, 0)
})

test('recommendations require editorial selection and collection references share the same project', () => {
  const excluded = { ...catalog.projects[0]!, featured: false }
  assert.deepEqual(filterProjects([excluded], 'recommend'), [])
  assert.deepEqual(filterProjects([excluded], 'all', excluded.name), [excluded], 'global search also finds projects outside recommendations')
  assert.equal(findProject('EXCALIDRAW/EXCALIDRAW'), catalog.projects.find(item => item.full_name === 'excalidraw/excalidraw'))
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
