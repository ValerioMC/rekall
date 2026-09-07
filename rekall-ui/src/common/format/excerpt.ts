const DEFAULT_LIMIT = 110

export function excerpt(body: string, limit: number = DEFAULT_LIMIT): string {
  const flat = body
    .replace(/[#`>*|_-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
  if (flat.length <= limit) return flat

  const cut = flat.slice(0, limit)
  const lastSpace = cut.lastIndexOf(' ')
  const kept = lastSpace > limit * 0.66 ? cut.slice(0, lastSpace) : cut
  return `${kept.trimEnd()}…`
}
