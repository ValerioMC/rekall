import type { SearchHit } from '@/features/search/model/search'

export const PALETTE_GROUPS = ['Go to', 'Projects', 'Tasks', 'Notes', 'In the text'] as const

export type PaletteGroup = (typeof PALETTE_GROUPS)[number]

/** Where an entry leads: a route, or a place in the text the server found, which also brings its pane forward. */
export type PaletteTarget =
  | { readonly kind: 'path'; readonly to: string }
  | { readonly kind: 'hit'; readonly hit: SearchHit; readonly to: string }

export type PaletteMark =
  | { readonly kind: 'screen' }
  | { readonly kind: 'project'; readonly projectId: string; readonly icon: string; readonly imageVersion: string | null }
  | { readonly kind: 'task'; readonly needsReview: boolean }
  | { readonly kind: 'note' }
  | { readonly kind: 'text'; readonly label: string }

export interface PaletteEntry {
  readonly key: string
  readonly group: PaletteGroup
  readonly title: string
  readonly detail: string
  readonly mark: PaletteMark
  readonly target: PaletteTarget
}

export interface PaletteSection {
  readonly group: PaletteGroup
  readonly entries: readonly PaletteEntry[]
}
