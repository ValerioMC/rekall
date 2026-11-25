export interface ArgumentLine {
  readonly key: string
  readonly value: string
}

/** The arguments of an MCP call as lines to read; text that is not a JSON object is one line with no key. */
export function mcpArguments(text: string): readonly ArgumentLine[] {
  const parsed = parseObject(text)
  if (!parsed) return text.trim() === '' ? [] : [{ key: '', value: text }]
  return Object.entries(parsed).map(([key, value]) => ({ key, value: readable(value) }))
}

function parseObject(text: string): Record<string, unknown> | null {
  try {
    const parsed: unknown = JSON.parse(text)
    const isObject = typeof parsed === 'object' && parsed !== null && !Array.isArray(parsed)
    return isObject ? (parsed as Record<string, unknown>) : null
  } catch {
    return null
  }
}

function readable(value: unknown): string {
  return typeof value === 'string' ? value : JSON.stringify(value)
}
