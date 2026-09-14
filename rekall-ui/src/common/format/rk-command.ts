export function rkCommand(anchor: string): string {
  return anchor ? `/rk ${anchor}` : ''
}

/**
 * The wrapup line, with an optional directive folded in. A directive is one line: newlines are
 * collapsed to spaces so it cannot submit the terminal line early, and `\`/`"` are escaped so it
 * cannot close the quote it sits in.
 */
export function rkWrapupCommand(anchor: string, directive = ''): string {
  const base = rkCommand(anchor)
  if (!base) return ''
  const flattened = directive.trim().replace(/\s+/g, ' ')
  if (!flattened) return `${base} wrapup`
  const escaped = flattened.replace(/\\/g, '\\\\').replace(/"/g, '\\"')
  return `${base} wrapup "${escaped}"`
}
