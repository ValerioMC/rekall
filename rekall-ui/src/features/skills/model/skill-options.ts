import type { ResourceSummary } from '@/features/claude-resources/model/claude-resources'
import { invocationName } from '@/features/skills/model/skill-assignment'

/** One skill the picker offers: the name a session invokes it by, and where it comes from. */
export interface SkillOption {
  readonly name: string
  readonly title: string
  readonly description: string | null
  /** The provider's name, as the picker's group header shows it. */
  readonly provider: string
}

export interface SkillOptionGroup {
  readonly provider: string
  readonly options: readonly SkillOption[]
}

const OWN_PROVIDER = 'Yours'

function optionOf(skill: ResourceSummary): SkillOption {
  const provider = skill.source && skill.source.kind !== 'own' ? skill.source.name : OWN_PROVIDER
  return { name: invocationName(skill), title: skill.name, description: skill.description, provider }
}

/** Every skill in the catalogue, once per invocation name, the person's own first. */
export function skillOptions(skills: readonly ResourceSummary[]): readonly SkillOption[] {
  const byName = new Map<string, SkillOption>()
  for (const skill of skills) {
    const option = optionOf(skill)
    if (!byName.has(option.name)) byName.set(option.name, option)
  }
  return [...byName.values()].sort(
    (a, b) => rank(a) - rank(b) || a.provider.localeCompare(b.provider) || a.title.localeCompare(b.title)
  )
}

function rank(option: SkillOption): number {
  return option.provider === OWN_PROVIDER ? 0 : 1
}

/** The options a query keeps: every word of it has to appear in the name, provider or description. */
export function filterSkillOptions(options: readonly SkillOption[], query: string): readonly SkillOption[] {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean)
  if (words.length === 0) return options
  return options.filter((option) => {
    const haystack = `${option.name} ${option.provider} ${option.description ?? ''}`.toLowerCase()
    return words.every((word) => haystack.includes(word))
  })
}

export function groupSkillOptions(options: readonly SkillOption[]): readonly SkillOptionGroup[] {
  const groups: SkillOptionGroup[] = []
  for (const option of options) {
    const last = groups[groups.length - 1]
    if (last && last.provider === option.provider)
      groups[groups.length - 1] = { ...last, options: [...last.options, option] }
    else groups.push({ provider: option.provider, options: [option] })
  }
  return groups
}
