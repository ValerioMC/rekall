/**
 * Where the database is, and every folder it has ever been. Mirrors
 * `dev.rekall.bootstrap.SettingsController` on the server.
 */
export type DatabaseSetupStatus = 'READY' | 'SETUP_NEEDED' | 'UNREACHABLE'

export interface DatabaseEntry {
  readonly id: string
  readonly label: string
  readonly path: string
  readonly active: boolean
  /** Whether the folder is on disk right now. A registered entry can stop being reachable. */
  readonly reachable: boolean
  readonly addedAt: string
  readonly lastUsedAt: string
}

export interface DatabaseStatus {
  readonly status: DatabaseSetupStatus
  readonly active: DatabaseEntry | null
  readonly databases: readonly DatabaseEntry[]
}

/** What the live-check endpoint reports about a path before anything commits to it. */
export interface FolderCheck {
  readonly resolvedPath: string
  readonly exists: boolean
  readonly isDirectory: boolean
  readonly writable: boolean
  readonly hasDatabase: boolean
  readonly usable: boolean
}
