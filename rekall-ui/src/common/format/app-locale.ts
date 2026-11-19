export const APP_LOCALE = 'en-GB'

const SYSTEM_USES_TWELVE_HOURS =
  new Intl.DateTimeFormat(undefined, { hour: 'numeric' }).resolvedOptions().hour12 === true

export function formatDate(date: Date): string {
  return date.toLocaleDateString(APP_LOCALE, { day: '2-digit', month: '2-digit', year: 'numeric' })
}

export function formatTimeOfDay(date: Date): string {
  return date.toLocaleTimeString(APP_LOCALE, {
    hour: '2-digit',
    minute: '2-digit',
    hour12: SYSTEM_USES_TWELVE_HOURS
  })
}

export function formatDateTime(date: Date): string {
  return `${formatDate(date)} ${formatTimeOfDay(date)}`
}
