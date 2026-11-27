import type { SelectOption } from '@/components/ui/AppSelect.vue'
import {
  CLAUDE_EFFORT_CHOICES,
  CLAUDE_MODEL_CHOICES,
  claudeEffortChoiceLabel,
  claudeModelChoiceLabel,
  claudeModelLabel,
  type ClaudeEffortChoice,
  type ClaudeModel
} from '@/features/claude/model/claude'

/** `declared`: the CLI lists the levels. `none`: it says the model takes none. `unreported`: it says nothing, so every level is offered. */
export type ClaudeEffortSupport =
  | { readonly kind: 'declared'; readonly levels: readonly ClaudeEffortChoice[] }
  | { readonly kind: 'none' }
  | { readonly kind: 'unreported' }

const ACCOUNT_DEFAULT = 'default'

export function claudeCatalogEntry(catalog: readonly ClaudeModel[] | null, value: string): ClaudeModel | null {
  return catalog?.find((model) => model.value === value) ?? null
}

export function claudeModelName(catalog: readonly ClaudeModel[] | null, value: string): string {
  const entry = value === ACCOUNT_DEFAULT ? null : claudeCatalogEntry(catalog, value)
  if (entry) return catalogName(entry)
  const alias = CLAUDE_MODEL_CHOICES.find((choice) => choice === value)
  return alias ? claudeModelChoiceLabel(alias) : value
}

/** The resolved id follows the provider's name only when that name does not say which version an alias starts. */
function catalogName({ value, displayName, resolvedModel }: ClaudeModel): string {
  if (!resolvedModel || resolvedModel === value) return displayName
  const version = claudeModelLabel(resolvedModel.replace('[1m]', '')) ?? resolvedModel
  return displayName.includes(version) ? displayName : `${displayName} · ${resolvedModel}`
}

/** Without a catalog the family aliases are offered. A saved value stays selectable either way. */
export function claudeModelOptions(catalog: readonly ClaudeModel[] | null, saved: string): readonly SelectOption[] {
  const offered: readonly string[] = catalog?.map((model) => model.value) ?? CLAUDE_MODEL_CHOICES
  const values = new Set([ACCOUNT_DEFAULT, ...offered, saved])
  return [...values].map((value) => ({ value, label: claudeModelName(catalog, value) }))
}

export function claudeEffortSupport(entry: ClaudeModel | null): ClaudeEffortSupport {
  if (entry?.supportsEffort === false) return { kind: 'none' }
  const listed = entry?.supportedEffortLevels
  if (!listed) return { kind: 'unreported' }
  const levels = CLAUDE_EFFORT_CHOICES.filter((level) => listed.includes(level))
  return levels.length ? { kind: 'declared', levels } : { kind: 'none' }
}

export function offersClaudeEffort(support: ClaudeEffortSupport, effort: ClaudeEffortChoice): boolean {
  if (effort === ACCOUNT_DEFAULT || support.kind === 'unreported') return true
  return support.kind === 'declared' && support.levels.includes(effort)
}

export function claudeEffortOptions(support: ClaudeEffortSupport, saved: ClaudeEffortChoice): readonly SelectOption[] {
  return CLAUDE_EFFORT_CHOICES.filter((level) => level === saved || offersClaudeEffort(support, level)).map((value) => ({
    value,
    label: claudeEffortChoiceLabel(value)
  }))
}
