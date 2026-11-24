export const TASK_PAGE_SIZE = 25

export interface PageOfRows<Row> {
  readonly rows: readonly Row[]
  readonly page: number
  readonly pageCount: number
  readonly total: number
  readonly from: number
  readonly to: number
}

/** The page asked for, held between the first and the last one that has rows. */
export function pageOf<Row>(rows: readonly Row[], requested: number, size: number = TASK_PAGE_SIZE): PageOfRows<Row> {
  const pageCount = Math.max(1, Math.ceil(rows.length / size))
  const page = Math.min(Math.max(1, requested), pageCount)
  const start = (page - 1) * size
  const pageRows = rows.slice(start, start + size)
  return { rows: pageRows, page, pageCount, total: rows.length, from: rows.length === 0 ? 0 : start + 1, to: start + pageRows.length }
}
