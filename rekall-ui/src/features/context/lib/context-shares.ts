import { tokensOf, type ContextSize } from '@/features/context/model/context'

export interface ContextShare {
  readonly label: string
  readonly tokens: number
  readonly percent: number
  readonly reference: boolean
}

const OTHERS_LABEL = 'Others'

/** The heaviest `limit - 1` parts of the context and the rest folded into one, each with its share of the characters. */
export function contextShares(size: ContextSize, limit: number): readonly ContextShare[] {
  const total = size.parts.reduce((sum, part) => sum + part.characters, 0)
  if (total === 0) return []
  const heaviest = [...size.parts].sort((left, right) => right.characters - left.characters)
  const shown = heaviest.length > limit ? heaviest.slice(0, limit - 1) : heaviest
  const rest = heaviest.slice(shown.length)
  const rows = shown.map((part) => ({
    label: part.label,
    characters: part.characters,
    reference: part.reference
  }))
  if (rest.length) {
    rows.push({ label: OTHERS_LABEL, characters: rest.reduce((sum, part) => sum + part.characters, 0), reference: false })
  }
  return rows.map((row) => ({
    label: row.label,
    tokens: tokensOf(row.characters),
    percent: (row.characters / total) * 100,
    reference: row.reference
  }))
}
