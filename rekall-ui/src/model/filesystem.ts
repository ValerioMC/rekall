/** One entry of a folder, with the absolute path the server resolved for it. */
export interface DirectoryEntry {
  readonly name: string
  readonly path: string
  readonly directory: boolean
  /** A dotfile, or a file the platform marks hidden. */
  readonly hidden: boolean
}

/** One crumb of the breadcrumb, from the filesystem root down to the listed folder. */
export interface PathSegment {
  readonly name: string
  readonly path: string
}

/**
 * One folder as the path picker shows it. Every path is absolute and already normalised by the
 * server, so nothing on this side joins, splits or resolves a path.
 */
export interface DirectoryListing {
  readonly path: string
  readonly parent: string | null
  readonly home: string
  readonly segments: readonly PathSegment[]
  /** Folders first, then files, each by name ignoring case. */
  readonly entries: readonly DirectoryEntry[]
  /** False when the folder exists but refused to be listed: `entries` is then empty. */
  readonly readable: boolean
  /** True when the folder held more entries than the server sends; a typed path still reaches past the cut. */
  readonly truncated: boolean
}
