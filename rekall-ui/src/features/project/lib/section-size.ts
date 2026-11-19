export const SECTION_BODY_MAX_CHARACTERS = 100_000

const CHARACTERS_PER_TOKEN = 4

/** A rough count: a session that reads the section pays about this many tokens for it. */
export function estimatedTokens(characters: number): number {
  return Math.ceil(characters / CHARACTERS_PER_TOKEN)
}

export function compactTokens(tokens: number): string {
  return tokens < 1000 ? String(tokens) : `${(tokens / 1000).toFixed(1)}k`
}

export function compactCharacters(characters: number): string {
  return characters < 10_000 ? characters.toLocaleString('en-US') : `${Math.round(characters / 1000)}k`
}
