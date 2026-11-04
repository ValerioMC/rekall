const MUTED_VERSION_KEY = 'rekall.update.mutedVersion'

/** The release the user asked not to be reminded of at startup; a newer release is announced again. */
export function mutedUpdateVersion(): string | null {
  try {
    return window.localStorage.getItem(MUTED_VERSION_KEY)
  } catch {
    return null
  }
}

export function muteUpdateVersion(version: string): void {
  try {
    window.localStorage.setItem(MUTED_VERSION_KEY, version)
  } catch {
    // Private windows refuse storage; the reminder just comes back at the next startup.
  }
}
