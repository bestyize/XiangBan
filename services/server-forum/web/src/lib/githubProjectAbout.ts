import translations from './githubAboutTranslations.json' with { type: 'json' }

type AboutSnapshot = {
  original: string
  zh: string
  observed_descriptions?: string[]
  icon_url?: string
  icon_fit?: string
}
const snapshots: Partial<Record<string, AboutSnapshot>> = translations
const normalized = (text: string) => text.trim().replace(/\s+/g, ' ')

export function curatedProjectAbout(fullName: string, fallback = '') {
  return snapshots[fullName.toLowerCase()]?.zh ?? fallback
}

/** Match the source text so a later About edit never inherits an old translation. */
export function repoAbout(fullName: string, original = '', translated = '') {
  const snapshot = snapshots[fullName.toLowerCase()]
  if (snapshot && [snapshot.original, ...(snapshot.observed_descriptions ?? [])]
    .some(text => normalized(text) === normalized(original))) {
    return { text: snapshot.zh, original: snapshot.original }
  }
  return { text: translated.trim() || original.trim() || '项目尚未填写 GitHub About。', original }
}

export function reviewedProjectIcon(fullName: string) {
  const snapshot = snapshots[fullName.toLowerCase()]
  return { url: snapshot?.icon_url, fit: snapshot?.icon_fit }
}
