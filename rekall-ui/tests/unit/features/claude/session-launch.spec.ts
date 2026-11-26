import { beforeEach, describe, expect, it } from 'vitest'
import { setPreferredEffort, setPreferredModel, setPreferredSessionKind, setSkipsPermissions } from '@/common/config/claude-launch'
import { launchChoices, setCodexBypass, setCodexEffort, setCodexModel, setPreferredProvider } from '@/common/config/session-launch'
import { installStorage } from '../../support/memoryStorage'

describe('session provider choices', () => {
  beforeEach(installStorage)

  it('uses the existing Claude defaults and keeps independent Codex preferences', () => {
    setPreferredModel('sonnet')
    setPreferredEffort('high')
    setPreferredSessionKind('chat')
    setSkipsPermissions(true)
    expect(launchChoices()).toEqual({ provider: 'CLAUDE', model: 'sonnet', effort: 'high', kind: 'CHAT', skipPermissions: true })

    setPreferredProvider('CODEX')
    expect(launchChoices()).toEqual({ provider: 'CODEX', model: 'default', effort: 'default', kind: 'TERMINAL', skipPermissions: false })
    setCodexModel('gpt-test')
    setCodexEffort('xhigh')
    setCodexBypass(true)
    expect(launchChoices('CLAUDE')).toEqual({ provider: 'CLAUDE', model: 'sonnet', effort: 'high', kind: 'CHAT', skipPermissions: true })
    setPreferredProvider('CLAUDE')
    setSkipsPermissions(false)
    expect(launchChoices()).toEqual({ provider: 'CLAUDE', model: 'sonnet', effort: 'high', kind: 'CHAT', skipPermissions: false })

    setPreferredProvider('CODEX')
    expect(launchChoices()).toEqual({ provider: 'CODEX', model: 'gpt-test', effort: 'xhigh', kind: 'TERMINAL', skipPermissions: true })
  })
})
