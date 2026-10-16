import { matchesQuery, type ResourceSummary, type SkillPlacement } from '@/features/claude-resources/model/claude-resources'

/** Skills the person keeps together, or the ones not put anywhere. */
export interface SkillBucket {
  readonly id: string
  /** The group's name; null for the skills in no group. */
  readonly label: string | null
  readonly skills: readonly ResourceSummary[]
}

/** Skills that share a provider: the person's own, or one source of read-only skills. */
export interface SkillShelf {
  readonly id: string
  readonly title: string
  readonly caption: string | null
  /** True for skills Claude or a plugin owns: they can be read but not changed. */
  readonly locked: boolean
  readonly buckets: readonly SkillBucket[]
  readonly count: number
}

const SYNC_SOURCE_LABEL: Readonly<Record<string, string>> = {
  anthropic: 'Made by Anthropic',
  'anthropic-example': 'Anthropic examples',
  plugin: 'From plugins'
}

export const OWN_SHELF_ID = 'yours'

function isOwn(skill: ResourceSummary): boolean {
  return skill.source ? skill.source.kind === 'own' : skill.editable
}

function ownShelf(skills: readonly ResourceSummary[], placements: readonly SkillPlacement[]): SkillShelf | null {
  if (skills.length === 0) return null
  const groupOf = new Map(placements.map((placement) => [placement.skillPath, placement.group]))
  const byGroup = new Map<string, ResourceSummary[]>()
  const loose: ResourceSummary[] = []
  for (const skill of skills) {
    const group = groupOf.get(skill.path)
    if (group === undefined) loose.push(skill)
    else byGroup.set(group, [...(byGroup.get(group) ?? []), skill])
  }
  const grouped: SkillBucket[] = [...byGroup.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, members]) => ({ id: `group:${label}`, label, skills: members }))
  const buckets = loose.length > 0 ? [...grouped, { id: 'loose', label: null, skills: loose }] : grouped
  return { id: OWN_SHELF_ID, title: 'Yours', caption: 'Files you can edit', locked: false, buckets, count: skills.length }
}

function providedShelves(skills: readonly ResourceSummary[]): readonly SkillShelf[] {
  const byShelf = new Map<string, { title: string; caption: string | null; skills: ResourceSummary[]; order: number }>()
  for (const skill of skills) {
    const source = skill.source
    const synced = source?.kind === 'synced'
    const id = synced ? `claude.ai:${source.detail ?? ''}` : `plugin:${source?.name ?? 'unknown'}`
    const shelf = byShelf.get(id) ?? {
      title: source?.name ?? 'Provided',
      caption: synced ? (SYNC_SOURCE_LABEL[source.detail ?? ''] ?? source.detail) : (source?.detail ?? null),
      skills: [],
      order: synced ? 0 : 1
    }
    shelf.skills.push(skill)
    byShelf.set(id, shelf)
  }
  return [...byShelf.entries()]
    .sort(([, a], [, b]) => a.order - b.order || a.title.localeCompare(b.title) || (a.caption ?? '').localeCompare(b.caption ?? ''))
    .map(([id, shelf]) => ({
      id,
      title: shelf.title,
      caption: shelf.caption,
      locked: true,
      buckets: [{ id: `${id}:all`, label: null, skills: shelf.skills }],
      count: shelf.skills.length
    }))
}

/** The skills that match the filter, laid out by who provides them: yours first, then read-only sources. */
export function buildSkillShelves(
  skills: readonly ResourceSummary[],
  placements: readonly SkillPlacement[],
  query: string
): readonly SkillShelf[] {
  const shown = skills.filter((skill) => matchesQuery(skill, query))
  const own = ownShelf(shown.filter(isOwn), placements)
  return [...(own ? [own] : []), ...providedShelves(shown.filter((skill) => !isOwn(skill)))]
}

/** The group names in use, for choosing where to move a skill. */
export function groupLabels(placements: readonly SkillPlacement[]): readonly string[] {
  return [...new Set(placements.map((placement) => placement.group))].sort((a, b) => a.localeCompare(b))
}
