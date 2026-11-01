import { test } from 'node:test'
import { deepStrictEqual } from 'node:assert'
import { publicWorks } from '../src/lib/creation.ts'

test('only published works are shown, with trusted works first and stable within a tier', () => {
  const works = [
    { id: 1, status: 'published', trust_level: 2 },
    { id: 2, status: 'removed', trust_level: 0 },
    { id: 3, status: 'published', trust_level: 1 },
    { id: 4, status: 'published', trust_level: 0 },
    { id: 5, status: 'published', trust_level: 1 },
  ]
  deepStrictEqual(publicWorks(works).map(work => work.id), [4, 3, 5, 1])
  deepStrictEqual(works.map(work => work.id), [1, 2, 3, 4, 5])
})
