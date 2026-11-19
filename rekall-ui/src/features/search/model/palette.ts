import type { StatusPillTone } from '@/components/ui/StatusPill.vue'
import type { SearchHit } from '@/features/search/model/search'

export const PALETTE_GROUPS = ['Projects', 'Tasks', 'Notes', 'Go to', 'In the text'] as const

export type PaletteGroup = (typeof PALETTE_GROUPS)[number]

/** What the palette narrows to: every group at a few rows each, or one group in full. */
export type PaletteScope = 'all' | PaletteGroup

export const PALETTE_SCOPES: readonly PaletteScope[] = ['all', ...PALETTE_GROUPS]

/** Where an entry leads: a route, or a place in the text the server found, which also brings its pane forward. */
export type PaletteTarget =
  | { readonly kind: 'path'; readonly to: string }
  | { readonly kind: 'hit'; readonly hit: SearchHit; readonly to: string }

export type PaletteMark =
  | { readonly kind: 'screen' }
  | { readonly kind: 'project'; readonly projectId: string; readonly icon: string; readonly imageVersion: string | null }
  | { readonly kind: 'task' }
  | { readonly kind: 'note' }
  | { readonly kind: 'text'; readonly label: string }

/** The project a task, note or text hit belongs to, shown beside it so a result says where it is. */
export interface PaletteOwner {
  readonly projectId: string
  readonly icon: string
  readonly imageVersion: string | null
  readonly label: string
}

export interface PaletteStatus {
  readonly label: string
  readonly tone: StatusPillTone
}

export interface PaletteEntry {
  readonly key: string
  readonly group: PaletteGroup
  readonly title: string
  readonly subtitle: string
  readonly anchor: string | null
  readonly status: PaletteStatus | null
  readonly excerpt: string | null
  readonly owner: PaletteOwner | null
  readonly mark: PaletteMark
  readonly target: PaletteTarget
}

/** A heading with its rows. `total` is how many the query found, of which `entries` are the ones shown. */
export interface PaletteSection {
  readonly group: PaletteGroup
  readonly label: string
  readonly entries: readonly PaletteEntry[]
  readonly total: number
}

/** How many each group found; `null` for the text before it is searched. */
export type PaletteCounts = Readonly<Record<PaletteGroup, number | null>>

export interface PaletteResults {
  readonly sections: readonly PaletteSection[]
  readonly counts: PaletteCounts
}
