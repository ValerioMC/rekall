/** A set of ids remembered per browser, such as the navigator's folded cards. */
export function loadPersistedSet<T extends string>(key: string): Set<T> {
  try {
    const raw = window.localStorage.getItem(key)
    const parsed: unknown = raw ? JSON.parse(raw) : []
    return Array.isArray(parsed) ? new Set(parsed.filter((id): id is T => typeof id === 'string')) : new Set()
  } catch {
    return new Set()
  }
}

// Storage can be full or blocked; the fold still applies for this session, it is just not kept.
export function savePersistedSet(key: string, ids: ReadonlySet<string>): void {
  try {
    window.localStorage.setItem(key, JSON.stringify([...ids]))
  } catch {
    return
  }
}

/** A copy of `ids` with `id` added when it was absent and removed when it was there. */
export function toggledSet<T>(ids: ReadonlySet<T>, id: T): Set<T> {
  const next = new Set(ids)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  return next
}
