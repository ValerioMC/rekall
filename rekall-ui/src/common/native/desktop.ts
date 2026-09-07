export interface ClaudeCodeLaunch {
  readonly directory: string
  readonly anchors: string
  readonly skipPermissions: boolean
}

export interface DesktopHost {
  pickFolder(currentPath: string): Promise<string | null>

  openInClaudeCode?(launch: ClaudeCodeLaunch): Promise<string>
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
