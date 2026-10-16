import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import {
  preferredEffort,
  preferredModel,
  setPreferredEffort,
  setPreferredModel
} from '@/common/config/claude-launch'

/**
 * The launch preferences live in the browser, read back on every "Run here". This checks the
 * model pick round-trips and that a missing or junk value reads as the account default rather
 * than throwing.
 */
function installStorage(): Map<string, string> {
  const values = new Map<string, string>()
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => void values.set(key, value),
      removeItem: (key: string) => void values.delete(key),
      clear: () => values.clear()
    }
  })
  return values
}

describe('preferredModel', () => {
  let storage: Map<string, string>

  beforeEach(() => {
    storage = installStorage()
  })

  afterEach(() => {
    delete (window as { localStorage?: unknown }).localStorage
  })

  it('defaults to the account setting when nothing is stored', () => {
    expect(preferredModel()).toBe('default')
  })

  it('round-trips a chosen model', () => {
    setPreferredModel('opus')
    expect(storage.get('rekall.claude.model')).toBe('opus')
    expect(preferredModel()).toBe('opus')
  })

  it('falls back to the account default for an unknown stored value', () => {
    storage.set('rekall.claude.model', 'gpt-5')
    expect(preferredModel()).toBe('default')
  })
})

describe('preferredEffort', () => {
  let storage: Map<string, string>

  beforeEach(() => {
    storage = installStorage()
  })

  afterEach(() => {
    delete (window as { localStorage?: unknown }).localStorage
  })

  it('defaults to unset when nothing is stored', () => {
    expect(preferredEffort()).toBe('default')
  })

  it('round-trips a chosen level', () => {
    setPreferredEffort('xhigh')
    expect(storage.get('rekall.claude.effort')).toBe('xhigh')
    expect(preferredEffort()).toBe('xhigh')
  })

  it('falls back to the default for a level the CLI does not define', () => {
    storage.set('rekall.claude.effort', 'ultra')
    expect(preferredEffort()).toBe('default')
  })
})
