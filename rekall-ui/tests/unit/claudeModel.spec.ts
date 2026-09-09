import { describe, expect, it } from 'vitest'
import {
  claudeEffortChoiceLabel,
  claudeEffortLabel,
  claudeModelChoiceLabel,
  claudeModelLabel
} from '@/model/claude'

describe('claudeModelLabel', () => {
  it('reads a concrete Claude Code model id as a family and version, whatever the generation', () => {
    expect(claudeModelLabel('claude-sonnet-5')).toBe('Sonnet 5')
    expect(claudeModelLabel('claude-sonnet-4-5-20250929')).toBe('Sonnet 4.5')
    expect(claudeModelLabel('claude-opus-5')).toBe('Opus 5')
    expect(claudeModelLabel('claude-fable-5-1')).toBe('Fable 5.1')
    expect(claudeModelLabel('claude-3-5-haiku-20241022')).toBe('Haiku')
  })

  it('reads a bare alias', () => {
    expect(claudeModelLabel('opus')).toBe('Opus')
    expect(claudeModelLabel('sonnet')).toBe('Sonnet')
    expect(claudeModelLabel('fable')).toBe('Fable')
  })

  it('marks the 1M-context variant', () => {
    expect(claudeModelLabel('sonnet[1m]')).toBe('Sonnet (1M)')
  })

  it('returns null when there is no model, and passes an unrecognised string through', () => {
    expect(claudeModelLabel(null)).toBeNull()
    expect(claudeModelLabel(undefined)).toBeNull()
    expect(claudeModelLabel('')).toBeNull()
    expect(claudeModelLabel('gpt-4o')).toBe('gpt-4o')
  })
})

describe('claudeModelChoiceLabel', () => {
  it('names each choice for the settings picker', () => {
    expect(claudeModelChoiceLabel('default')).toBe('Account default')
    expect(claudeModelChoiceLabel('sonnet')).toBe('Sonnet')
    expect(claudeModelChoiceLabel('fable')).toBe('Fable')
    expect(claudeModelChoiceLabel('opus')).toBe('Opus')
    expect(claudeModelChoiceLabel('haiku')).toBe('Haiku')
  })
})

describe('effort labels', () => {
  it('names each effort choice for the settings picker', () => {
    expect(claudeEffortChoiceLabel('default')).toBe('Account default')
    expect(claudeEffortChoiceLabel('low')).toBe('Low')
    expect(claudeEffortChoiceLabel('high')).toBe('High')
    expect(claudeEffortChoiceLabel('xhigh')).toBe('Extra-high')
    expect(claudeEffortChoiceLabel('max')).toBe('Max')
  })

  it('reads a recorded effort level, null when the session ran at the default', () => {
    expect(claudeEffortLabel('high')).toBe('High')
    expect(claudeEffortLabel('xhigh')).toBe('Extra-high')
    expect(claudeEffortLabel(null)).toBeNull()
    expect(claudeEffortLabel('')).toBeNull()
    expect(claudeEffortLabel('weird')).toBe('weird')
  })
})
