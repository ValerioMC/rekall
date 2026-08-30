/**
 * What the macOS application adds to the page, and nothing more.
 *
 * The same bundle runs in a browser through `make run` and inside Rekall.app, so everything here
 * is optional by construction: `desktopHost()` returns null in a browser and every caller has to
 * have a browser answer anyway. The bridge itself is installed by the app's launcher
 * (packaging/macos/Launcher.swift) as a script that runs before the page does.
 */
export interface DesktopHost {
  /**
   * Opens the system folder chooser and resolves with an absolute path, or null when the user
   * dismissed it. `currentPath` is where the panel opens, when it points somewhere real.
   */
  pickFolder(currentPath: string): Promise<string | null>
}

declare global {
  interface Window {
    rekallDesktop?: DesktopHost
  }
}

export function desktopHost(): DesktopHost | null {
  return typeof window.rekallDesktop?.pickFolder === 'function' ? window.rekallDesktop : null
}

/**
 * A cancelled panel and a bridge that failed are the same thing to a caller: no path was chosen,
 * carry on with what is typed. Nothing here is allowed to break the field it is attached to.
 */
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
