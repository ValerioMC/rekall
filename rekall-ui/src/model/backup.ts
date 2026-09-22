/** Why a backup was taken; it is the last part of the file's name. */
export type BackupReason = 'AUTO' | 'MANUAL' | 'BEFORE_RESTORE' | 'UPLOADED'

export const BACKUP_REASON_LABEL: Readonly<Record<BackupReason, string>> = {
  AUTO: 'automatic',
  MANUAL: 'taken by hand',
  BEFORE_RESTORE: 'before a restore',
  UPLOADED: 'uploaded'
}

/** One zip of the whole database in the `backups` folder beside it. */
export interface BackupFile {
  readonly name: string
  readonly sizeBytes: number
  readonly createdAt: string
  readonly reason: BackupReason
}

export interface BackupStatus {
  /** False for a database that is not a file (in memory), which has no backups. */
  readonly available: boolean
  readonly folder: string | null
  readonly intervalHours: number
  readonly keep: number
  /** Newest first. */
  readonly backups: readonly BackupFile[]
  readonly lastFailure: string | null
}

/** 1 536 → "1.5 KB". */
export function readableSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  const units = ['KB', 'MB', 'GB']
  let value = bytes / 1024
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024
    unit += 1
  }
  return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unit]}`
}
