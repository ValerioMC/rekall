/** The fixed set of glowing icons a tag can wear. Chosen from here, never free-form, so the
 *  console always knows how to draw every badge it stores. */
export const TAG_ICON_KEYS = [
  'star',
  'bolt',
  'flame',
  'shield',
  'gem',
  'heart',
  'target',
  'compass',
  'flag',
  'crown'
] as const

export type TagIconKey = (typeof TAG_ICON_KEYS)[number]

export const TAG_ICON_LABEL: Readonly<Record<TagIconKey, string>> = {
  star: 'Star',
  bolt: 'Bolt',
  flame: 'Flame',
  shield: 'Shield',
  gem: 'Gem',
  heart: 'Heart',
  target: 'Target',
  compass: 'Compass',
  flag: 'Flag',
  crown: 'Crown'
}

/** The fixed palette a tag's glow is picked from. Each key has a `--color-tag-<key>` triplet
 *  (base/soft/line) in `main.css`, the same shape `identityHue` reads for identity dots. */
export const TAG_COLOR_KEYS = [
  'crimson',
  'tangerine',
  'citrine',
  'moss',
  'teal',
  'cerulean',
  'violet',
  'magenta'
] as const

export type TagColorKey = (typeof TAG_COLOR_KEYS)[number]

export const TAG_COLOR_LABEL: Readonly<Record<TagColorKey, string>> = {
  crimson: 'Crimson',
  tangerine: 'Tangerine',
  citrine: 'Citrine',
  moss: 'Moss',
  teal: 'Teal',
  cerulean: 'Cerulean',
  violet: 'Violet',
  magenta: 'Magenta'
}

export interface TagColor {
  readonly base: string
  readonly soft: string
  readonly line: string
}

export function tagColor(key: string): TagColor {
  const known = (TAG_COLOR_KEYS as readonly string[]).includes(key) ? key : 'crimson'
  return {
    base: `var(--color-tag-${known})`,
    soft: `var(--color-tag-${known}-soft)`,
    line: `var(--color-tag-${known}-line)`
  }
}

export function isTagIconKey(value: string): value is TagIconKey {
  return (TAG_ICON_KEYS as readonly string[]).includes(value)
}

export function isTagColorKey(value: string): value is TagColorKey {
  return (TAG_COLOR_KEYS as readonly string[]).includes(value)
}
