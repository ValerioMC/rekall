/**
 * The grid's column heads, Monday first, in the same locale as the month title above them and
 * every other date on the page. 1 January 2024 was a Monday, so it and the six days after it
 * name the columns in order.
 */
export function weekdayLabels(locale?: string): string[] {
  return Array.from({ length: 7 }, (_, i) =>
    capitalised(new Date(2024, 0, 1 + i).toLocaleDateString(locale, { weekday: 'short' }), locale)
  )
}

/**
 * A heading's first letter in capitals, in the given locale. Some locales name the days in lower
 * case (`lun`, `mar`), which reads as a typo at the head of a column; done here rather than with
 * `text-transform` so the string is the same wherever it lands, a label or a tooltip.
 */
export function capitalised(text: string, locale?: string): string {
  return text.charAt(0).toLocaleUpperCase(locale) + text.slice(1)
}

export function monthGridDays(year: number, month: number): Date[] {
  const firstOfMonth = new Date(year, month, 1)
  const mondayOffset = (firstOfMonth.getDay() + 6) % 7
  const start = new Date(year, month, 1 - mondayOffset)
  return Array.from(
    { length: 42 },
    (_, i) => new Date(start.getFullYear(), start.getMonth(), start.getDate() + i)
  )
}

export function isSameDay(a: Date, b: Date): boolean {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  )
}

export function isSameMonth(date: Date, year: number, month: number): boolean {
  return date.getFullYear() === year && date.getMonth() === month
}

export function dateKey(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`
}
