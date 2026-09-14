export type ReadAlign = 'left' | 'center'

export const READ_WIDTH_MIN = 480
export const READ_WIDTH_MAX = 920
export const READ_WIDTH_DEFAULT = 680

const WIDTH_KEY = 'rekall.read.width'
const ALIGN_KEY = 'rekall.read.align'

function clampWidth(value: number): number {
  if (!Number.isFinite(value)) return READ_WIDTH_DEFAULT
  return Math.min(READ_WIDTH_MAX, Math.max(READ_WIDTH_MIN, value))
}

/** The reading column's max-width in the Read mode of the markdown preview, shared across every pane. */
export function readModeWidth(): number {
  try {
    const stored = Number(window.localStorage.getItem(WIDTH_KEY))
    return stored ? clampWidth(stored) : READ_WIDTH_DEFAULT
  } catch {
    return READ_WIDTH_DEFAULT
  }
}

export function setReadModeWidth(value: number): void {
  try {
    window.localStorage.setItem(WIDTH_KEY, String(clampWidth(value)))
  } catch {
  }
}

/** Where the reading column sits: centered on the pane, or flush to the left with a fixed gutter. */
export function readModeAlign(): ReadAlign {
  try {
    const stored = window.localStorage.getItem(ALIGN_KEY)
    return stored === 'left' ? 'left' : 'center'
  } catch {
    return 'center'
  }
}

export function setReadModeAlign(value: ReadAlign): void {
  try {
    window.localStorage.setItem(ALIGN_KEY, value)
  } catch {
  }
}
