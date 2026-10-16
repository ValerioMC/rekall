/**
 * Mermaid's parser databases are classes whose getters read their own state, so they are called
 * on the database itself. This checks the getters a reader needs are there before it calls them.
 */
export type MermaidDb<Name extends string> = Readonly<Record<Name, () => unknown>>

export function requireGetters<Name extends string>(db: unknown, names: readonly Name[]): MermaidDb<Name> {
  if (typeof db !== 'object' || db === null) throw new Error('Mermaid returned no diagram model to read.')
  const missing = names.filter((name) => typeof (db as Record<string, unknown>)[name] !== 'function')
  if (missing.length > 0) throw new Error(`Mermaid's diagram model has no ${missing.join(', ')}.`)
  return db as MermaidDb<Name>
}

/** Mermaid keeps some collections in a Map; a plain record reads the same. */
export function asRecord(value: unknown): unknown {
  return value instanceof Map ? Object.fromEntries(value) : value
}
