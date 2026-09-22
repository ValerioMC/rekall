/**
 * A system notification, from whichever host the console runs in: the native one Rekall.app
 * installs as `window.rekallDesktop.notify`, or the browser's Notification API. Whether to notify
 * at all is a per-machine preference kept in localStorage, on unless turned off.
 */

import type { DesktopNotice } from './desktop'

export type { DesktopNotice }

const ENABLED_KEY = 'rekall.notify.claims'

export function claimNotificationsEnabled(): boolean {
  try {
    return window.localStorage.getItem(ENABLED_KEY) !== 'off'
  } catch {
    return true
  }
}

export function setClaimNotificationsEnabled(enabled: boolean): void {
  try {
    window.localStorage.setItem(ENABLED_KEY, enabled ? 'on' : 'off')
  } catch {
  }
}

function nativeNotify(): ((notice: DesktopNotice) => Promise<boolean>) | null {
  const notify = window.rekallDesktop?.notify
  return typeof notify === 'function' ? notify.bind(window.rekallDesktop) : null
}

/** Whether this host can show a notification at all, asked or not. */
export function canNotify(): boolean {
  return nativeNotify() !== null || typeof window.Notification === 'function'
}

/**
 * Asks the browser for permission, which it only grants from a click. The native host asks the
 * system itself the first time it posts, so there is nothing to ask for here.
 */
export async function requestNotificationPermission(): Promise<boolean> {
  if (nativeNotify()) return true
  if (typeof window.Notification !== 'function') return false
  if (window.Notification.permission === 'granted') return true
  if (window.Notification.permission === 'denied') return false
  return (await window.Notification.requestPermission()) === 'granted'
}

/** Posts the notice if this machine wants them and the host allows it; says whether it did. */
export async function postNotice(notice: DesktopNotice): Promise<boolean> {
  if (!claimNotificationsEnabled()) return false
  const native = nativeNotify()
  if (native) return native(notice)
  if (typeof window.Notification !== 'function' || window.Notification.permission !== 'granted') return false
  new window.Notification(notice.title, { body: notice.body, tag: notice.title })
  return true
}
