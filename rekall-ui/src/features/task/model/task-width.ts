import { TASK_WIDTH_PIXELS, type TaskWidth } from '@/common/config/ui-preferences'

export const TASK_WIDTH_LABEL: Readonly<Record<TaskWidth, string>> = {
  narrow: 'Narrow',
  standard: 'Standard',
  wide: 'Wide',
  full: 'Full'
}

/** The CSS `max-width` of the column; `full` takes the whole window. */
export function columnMaxWidth(width: TaskWidth): string {
  const pixels = TASK_WIDTH_PIXELS[width]
  return pixels === null ? 'none' : `${pixels}px`
}
