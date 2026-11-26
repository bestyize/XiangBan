import assert from 'node:assert/strict'
import test from 'node:test'
import { ADMIN_COMMANDS, ADMIN_NAV_GROUPS, adminSectionPath, resolveAdminRoute } from '../src/views/admin/adminNav.ts'

const LEGACY_SECTIONS = [
  '',
  'metrics',
  'reports',
  'content-reports',
  'topics',
  'users',
  'editor-accounts',
  'editor',
  'categories',
  'moderation',
  'words',
  'packs',
  'projects',
  'weekly',
  'discover',
  'open-source',
  'collection',
  'community-operations',
  'tool-mirrors',
  'community-editorial',
  'audit',
  'account',
  'contact',
]

test('admin sidebar collapses 23 legacy routes into 4 groups and 12 nav items', () => {
  assert.equal(ADMIN_NAV_GROUPS.length, 4)
  const navCount = ADMIN_NAV_GROUPS.reduce((sum, g) => sum + g.items.length, 0)
  assert.equal(navCount, 12)
  assert.equal(ADMIN_COMMANDS.length, 23)
})

test('every legacy /admin/:section route resolves cleanly without losing target section', () => {
  for (const section of LEGACY_SECTIONS) {
    const resolved = resolveAdminRoute(section)
    assert.equal(resolved.section, section, `legacy section "${section}" should stay addressable`)
    assert.ok(resolved.nav.label.length > 0)
  }
})

test('group id aliases resolve to the first subtab and unknown section falls back to overview', () => {
  assert.equal(resolveAdminRoute('review').section, 'reports')
  assert.equal(resolveAdminRoute('taxonomy').section, 'categories')
  assert.equal(resolveAdminRoute('works').section, 'packs')
  assert.equal(resolveAdminRoute('ops-sys').section, 'contact')
  assert.equal(resolveAdminRoute('non-existent-route').section, '')
  assert.equal(adminSectionPath(''), '/admin')
  assert.equal(adminSectionPath('reports'), '/admin/reports')
})
