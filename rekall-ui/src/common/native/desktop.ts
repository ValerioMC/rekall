export interface ClaudeCodeLaunch {
  readonly directory: string
  readonly anchors: string
  readonly skipPermissions: boolean
}

/** A system notification the native host posts. */
export interface DesktopNotice {
  readonly title: string
  readonly body: string
}

export interface DesktopHost {
  pickFolder(currentPath: string): Promise<string | null>

  openInClaudeCode?(launch: ClaudeCodeLaunch): Promise<string>

  /** Posts a system notification; resolves false when the system refused it. */
  notify?(notice: DesktopNotice): Promise<boolean>

  /** Stand in for the native titlebar buttons the window draws none of. */
  closeWindow?(): Promise<void>
  minimizeWindow?(): Promise<void>
  toggleMaximizeWindow?(): Promise<void>
}

declare global {
  interface Window {
    rekallDesktop?: DesktopHost
  }
}

export function canLaunchClaudeCode(): boolean {
  return typeof window.rekallDesktop?.openInClaudeCode === 'function'
}

export async function launchClaudeCode(launch: ClaudeCodeLaunch): Promise<string> {
  const host = window.rekallDesktop
  if (typeof host?.openInClaudeCode !== 'function') {
    throw new Error('This window cannot open a terminal.')
  }
  return host.openInClaudeCode(launch)
}

export function desktopHost(): DesktopHost | null {
  return typeof window.rekallDesktop?.pickFolder === 'function' ? window.rekallDesktop : null
}

/** True inside the Tauri window, false in a browser tab: the two share this frontend. */
export function isDesktopApp(): boolean {
  return typeof window.rekallDesktop !== 'undefined'
}

export async function closeWindow(): Promise<void> {
  await window.rekallDesktop?.closeWindow?.()
}

export async function minimizeWindow(): Promise<void> {
  await window.rekallDesktop?.minimizeWindow?.()
}

export async function toggleMaximizeWindow(): Promise<void> {
  await window.rekallDesktop?.toggleMaximizeWindow?.()
}

export async function pickFolder(currentPath: string): Promise<string | null> {
  const host = desktopHost()
  if (!host) return null
  try {
    const chosen = await host.pickFolder(currentPath)
    return typeof chosen === 'string' && chosen.length > 0 ? chosen : null
  } catch {
    return null
  }
}
