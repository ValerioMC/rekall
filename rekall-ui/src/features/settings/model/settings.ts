export type DatabaseSetupStatus = 'READY' | 'SETUP_NEEDED' | 'UNREACHABLE'

export interface DatabaseEntry {
  readonly id: string
  readonly label: string
  readonly path: string
  readonly active: boolean
  readonly reachable: boolean
  readonly addedAt: string
  readonly lastUsedAt: string
}

export interface DatabaseStatus {
  readonly status: DatabaseSetupStatus
  readonly active: DatabaseEntry | null
  readonly databases: readonly DatabaseEntry[]
}

export interface FolderCheck {
  readonly resolvedPath: string
  readonly exists: boolean
  readonly isDirectory: boolean
  readonly writable: boolean
  readonly hasDatabase: boolean
  readonly usable: boolean
}
