/** The fixed set of icons a project can wear, independent of its identity colour (which stays
 *  hash-derived from the project's id and is never user-chosen). Picked from here, never
 *  free-form, so the console always knows how to draw every project it stores. */
export const PROJECT_ICON_KEYS = [
  'folder',
  'rocket',
  'star',
  'bolt',
  'flame',
  'shield',
  'gem',
  'heart',
  'target',
  'compass',
  'flag',
  'crown',
  'box',
  'briefcase',
  'beaker',
  'book',
  'bulb',
  'camera',
  'chart',
  'cloud',
  'code',
  'cube',
  'database',
  'gear',
  'globe',
  'key',
  'layers',
  'map',
  'palette',
  'grid',
  'terminal',
  'wrench'
] as const

export type ProjectIconKey = (typeof PROJECT_ICON_KEYS)[number]

export const PROJECT_ICON_LABEL: Readonly<Record<ProjectIconKey, string>> = {
  folder: 'Folder',
  rocket: 'Rocket',
  star: 'Star',
  bolt: 'Bolt',
  flame: 'Flame',
  shield: 'Shield',
  gem: 'Gem',
  heart: 'Heart',
  target: 'Target',
  compass: 'Compass',
  flag: 'Flag',
  crown: 'Crown',
  box: 'Box',
  briefcase: 'Briefcase',
  beaker: 'Beaker',
  book: 'Book',
  bulb: 'Bulb',
  camera: 'Camera',
  chart: 'Chart',
  cloud: 'Cloud',
  code: 'Code',
  cube: 'Cube',
  database: 'Database',
  gear: 'Gear',
  globe: 'Globe',
  key: 'Key',
  layers: 'Layers',
  map: 'Map',
  palette: 'Palette',
  grid: 'Grid',
  terminal: 'Terminal',
  wrench: 'Wrench'
}

export function isProjectIconKey(value: string): value is ProjectIconKey {
  return (PROJECT_ICON_KEYS as readonly string[]).includes(value)
}

export function projectIconKey(value: string): ProjectIconKey {
  return isProjectIconKey(value) ? value : 'folder'
}
