import type { Ref } from 'vue'

/** The list with the row of the same id swapped for `row`; a list without it comes back unchanged. */
export function replaceById<Row extends { id: unknown }>(rows: readonly Row[], row: Row): Row[] {
  return rows.map((candidate) => (candidate.id === row.id ? row : candidate))
}

/** Replaces the row of the same id, or adds `row` at `end` of the list when there is none. */
export function upsertById<Row extends { id: unknown }>(
  rows: readonly Row[],
  row: Row,
  end: 'first' | 'last'
): Row[] {
  if (rows.some((candidate) => candidate.id === row.id)) return replaceById(rows, row)
  return end === 'first' ? [row, ...rows] : [...rows, row]
}

/** Text left blank is stored as none. */
export function blankToNull(text: string): string | null {
  return text.trim() === '' ? null : text
}

/**
 * Runs one write the editors autosave, and keeps the save indicator honest: `saving` while it
 * runs, then `saved`, or `unsaved` with the failure passed on.
 */
export async function trackSave<Result>(
  saveState: Ref<string>,
  write: () => Promise<Result>
): Promise<Result> {
  saveState.value = 'saving'
  try {
    const result = await write()
    saveState.value = 'saved'
    return result
  } catch (error) {
    saveState.value = 'unsaved'
    throw error
  }
}
