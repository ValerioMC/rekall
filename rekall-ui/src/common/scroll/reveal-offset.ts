export interface VerticalSpan {
  readonly top: number
  readonly bottom: number
}

/**
 * How far a scroller must move to bring `row` to the middle of `viewport`, or `null` when the row
 * is already fully inside it. The browser clamps the move, so a short list that cannot centre the
 * row simply ends with it visible.
 */
export function revealOffset(row: VerticalSpan, viewport: VerticalSpan): number | null {
  if (row.top >= viewport.top && row.bottom <= viewport.bottom) return null
  const rowCentre = (row.top + row.bottom) / 2
  const viewportCentre = (viewport.top + viewport.bottom) / 2
  return rowCentre - viewportCentre
}
