/** One part of a task's context, as the size popover lists it. */
export interface ContextSizePart {
  readonly label: string
  readonly characters: number
  /** A note handed over as a reference: a line and an anchor, not its body. */
  readonly reference: boolean
}

/** What `/rk project:… task:…` hands a session for this task, and roughly what it costs. */
export interface ContextSize {
  readonly characters: number
  readonly estimatedTokens: number
  readonly parts: readonly ContextSizePart[]
}

/** The same characters-per-token the server estimates with, so a part and the total agree. */
export const CHARACTERS_PER_TOKEN = 3.5

export function tokensOf(characters: number): number {
  return Math.ceil(characters / CHARACTERS_PER_TOKEN)
}

/** 842 → "842", 12 400 → "12.4k". */
export function compactCount(value: number): string {
  if (value < 1000) return String(value)
  const thousands = value / 1000
  return `${thousands >= 100 ? Math.round(thousands) : thousands.toFixed(1).replace(/\.0$/, '')}k`
}
