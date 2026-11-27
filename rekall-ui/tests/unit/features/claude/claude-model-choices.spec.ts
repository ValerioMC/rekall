import { describe, expect, it } from 'vitest'
import {
  claudeCatalogEntry,
  claudeEffortOptions,
  claudeEffortSupport,
  claudeModelName,
  claudeModelOptions,
  offersClaudeEffort
} from '@/features/claude/lib/claude-model-choices'
import type { ClaudeModel } from '@/features/claude/model/claude'

function entry(overrides: Partial<ClaudeModel> = {}): ClaudeModel {
  return { value: 'sonnet', displayName: 'Sonnet', resolvedModel: null, supportsEffort: null, supportedEffortLevels: null, ...overrides }
}

describe('the offered Claude Code models', () => {
  it('offers the provider values under its names, with one Account default in place of its own', () => {
    const catalog = [entry({ value: 'default', displayName: 'Default (recommended)' }), entry({ value: 'sonnet', displayName: 'Sonnet 5' })]

    expect(claudeModelOptions(catalog, 'default')).toEqual([
      { value: 'default', label: 'Account default' },
      { value: 'sonnet', label: 'Sonnet 5' }
    ])
  })

  it('adds the resolved version to an alias only when its name does not say it', () => {
    const named = entry({ displayName: 'Sonnet 5.5 (1M context)', value: 'sonnet[1m]', resolvedModel: 'claude-sonnet-5-5[1m]' })
    const unnamed = entry({ resolvedModel: 'claude-sonnet-5-5' })
    const versioned = entry({ value: 'claude-sonnet-5', displayName: 'Previous Sonnet', resolvedModel: 'claude-sonnet-5' })

    expect(claudeModelName([named], 'sonnet[1m]')).toBe('Sonnet 5.5 (1M context)')
    expect(claudeModelName([unnamed], 'sonnet')).toBe('Sonnet · claude-sonnet-5-5')
    expect(claudeModelName([versioned], 'claude-sonnet-5')).toBe('Previous Sonnet')
  })

  it('names an unlisted value by its alias or as it is spelled', () => {
    expect(claudeModelName([], 'fable')).toBe('Fable')
    expect(claudeModelName(null, 'claude-example-7[1m]')).toBe('claude-example-7[1m]')
  })
})

describe('the efforts of a Claude Code model', () => {
  it('offers the declared levels a launch accepts', () => {
    const support = claudeEffortSupport(entry({ supportsEffort: true, supportedEffortLevels: ['high', 'ultra', 'low'] }))

    expect(support).toEqual({ kind: 'declared', levels: ['low', 'high'] })
    expect(offersClaudeEffort(support, 'max')).toBe(false)
  })

  it('tells a model that takes none from a CLI that says nothing', () => {
    expect(claudeEffortSupport(entry({ supportsEffort: false }))).toEqual({ kind: 'none' })
    expect(claudeEffortSupport(entry())).toEqual({ kind: 'unreported' })
    expect(claudeEffortSupport(claudeCatalogEntry(null, 'sonnet'))).toEqual({ kind: 'unreported' })
    expect(claudeEffortOptions({ kind: 'unreported' }, 'default')).toHaveLength(6)
  })

  it('keeps the saved level selectable when the model does not offer it', () => {
    expect(claudeEffortOptions({ kind: 'none' }, 'high').map(({ value }) => value)).toEqual(['default', 'high'])
  })
})
