export interface PathCursor {
  /** The group whose contents are listed: empty for the connection's root path. */
  readonly parent: string
  /** What has been typed of a name inside it. */
  readonly fragment: string
}

/** The path inside a pasted address: the scheme, host, `.git` suffix and outer slashes are dropped. */
export function pathOf(raw: string): string {
  const typed = raw.trim()
  const address = typed.replace(/^https?:\/\/[^/]*\/?/, '')
  return address.replace(/^\/+|\/+$/g, '').replace(/\.git$/, '')
}

/** Where a typed path is in the tree: the group it names up to its last slash, and the name begun after it. */
export function cursorOf(typed: string): PathCursor {
  const path = pathOf(typed)
  if (typed.trim().endsWith('/')) return { parent: path, fragment: '' }
  const slash = path.lastIndexOf('/')
  return slash < 0 ? { parent: '', fragment: path } : { parent: path.slice(0, slash), fragment: path.slice(slash + 1) }
}

/** The path of each group above the last one named, from the top: what a breadcrumb jumps back to. */
export function ancestorsOf(parent: string): readonly { readonly name: string; readonly path: string }[] {
  const names = parent.split('/').filter(Boolean)
  return names.map((name, index) => ({ name, path: names.slice(0, index + 1).join('/') }))
}

/** The last segment of a path, the name a list row shows under its group. */
export function leafOf(path: string): string {
  return path.slice(path.lastIndexOf('/') + 1)
}

/** The part of a name that matches what was typed, so a row can set it in bold. */
export function splitOnMatch(name: string, fragment: string): readonly [string, string, string] {
  const at = fragment ? name.toLowerCase().indexOf(fragment.toLowerCase()) : -1
  if (at < 0) return [name, '', '']
  return [name.slice(0, at), name.slice(at, at + fragment.length), name.slice(at + fragment.length)]
}
