export const UPDATE_CHECKS = ['UP_TO_DATE', 'UPDATE_AVAILABLE', 'UNAVAILABLE', 'DISABLED'] as const
export type UpdateCheck = (typeof UPDATE_CHECKS)[number]

export interface LatestRelease {
  readonly version: string
  readonly releaseUrl: string
  /** The asset for this platform; null when the release carries none. */
  readonly downloadUrl: string | null
}

export interface VersionStatus {
  readonly current: string
  readonly check: UpdateCheck
  readonly latest: LatestRelease | null
}
