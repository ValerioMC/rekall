/**
 * What the macOS application adds to the page, and nothing more.
 *
 * The same bundle runs in a browser through `make run` and inside Rekall.app, so everything here
 * is optional by construction: `desktopHost()` returns null in a browser and every caller has to
 * have a browser answer anyway. The bridge itself is installed by the app's launcher
 * (packaging/macos/Launcher.swift) as a script that runs before the page does.
 */

/** A terminal to open, said in the only three things the native side accepts. */
export interface ClaudeCodeLaunch {
  /** Absolute path the session starts in, which is the one thing it cannot be told later. */
  readonly directory: string
  /** The anchors exactly as `/rk` takes them, and nothing that means anything to a shell. */
  readonly anchors: string
  readonly skipPermissions: boolean
}

export interface DesktopHost {
  /**
   * Opens the system folder chooser and resolves with an absolute path, or null when the user
   * dismissed it. `currentPath` is where the panel opens, when it points somewhere real.
   */
  pickFolder(currentPath: string): Promise<string | null>

  /**
   * Opens a terminal in `directory` already running Claude Code on those anchors, and resolves
   * with the name of the terminal it used. Optional: a window from a build before this existed
   * has the chooser and not this, and a page that assumed otherwise would throw on a click.
   */
  openInClaudeCode?(launch: ClaudeCodeLaunch): Promise<string>
}

declare global {
  interface Window {
    rekallDesktop?: DesktopHost
  }
}

/** Fixed for the lifetime of the page, like the chooser: the bridge is installed before it. */
export function canLaunchClaudeCode(): boolean {
  return typeof window.rekallDesktop?.openInClaudeCode === 'function'
}

/**
 * Opens the terminal, or throws with the reason.
 *
 * <p>Louder than {@link pickFolder} on purpose. A dismissed chooser leaves the field exactly as
 * it was, so silence is the right answer there; a launch that does not happen changes nothing on
 * screen either, and silence would read as a button that does nothing.
 */
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
