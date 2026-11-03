import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ClaudeCodeSection from '@/features/settings/components/ClaudeCodeSection.vue'
import QueueSettingsAside from '@/features/queue/components/QueueSettingsAside.vue'
import { preferredSessionKind } from '@/common/config/claude-launch'
import { installStorage } from '../../support/memoryStorage'

vi.mock('@/features/claude/api/claude.api', () => ({
  fetchClaudeInstallation: vi.fn().mockResolvedValue({
    status: 'CONNECTED',
    endpoint: 'http://localhost:47355/mcp',
    registeredUrl: 'http://localhost:47355/mcp',
    folderScoped: [],
    commandInstalled: true,
    cliPath: '/bin/claude',
    manualCommand: ''
  }),
  installClaudeIntegration: vi.fn(),
  fetchClaudeUsage: vi.fn().mockResolvedValue({ status: 'UNAVAILABLE', limits: [], fetchedAt: '' })
}))

describe('Settings > Claude Code > Session view', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
  })

  it('offers the terminal and the chat, marks the chat experimental, and keeps the choice', async () => {
    const section = mount(ClaudeCodeSection)
    await flushPromises()

    const terminal = section.get('[data-testid=claude-session-view-terminal]')
    const chat = section.get('[data-testid=claude-session-view-chat]')
    expect(terminal.attributes('aria-checked')).toBe('true')
    expect(chat.text()).toBe('Chat (experimental)')
    expect(section.get('[data-testid=claude-session-view]').text()).toContain('without a terminal')

    await chat.trigger('click')

    expect(chat.attributes('aria-checked')).toBe('true')
    expect(terminal.attributes('aria-checked')).toBe('false')
    expect(preferredSessionKind()).toBe('chat')
  })
})

describe('the run queue Session view', () => {
  it('is chosen in the queue settings and reaches the draft', async () => {
    const aside = mount(QueueSettingsAside, {
      props: {
        now: 0,
        startIso: null,
        startInPast: false,
        when: 'now',
        startLocal: '',
        ceilingOn: false,
        ceiling: 85,
        model: 'default',
        effort: 'default',
        skipPermissions: true,
        sessionKind: 'TERMINAL',
        parallelSessions: '1'
      }
    })

    const select = aside.get('#queue-session-view')
    expect(select.findAll('option').map((option) => option.text())).toEqual(['Terminal', 'Chat (experimental)'])

    await select.setValue('CHAT')

    expect(aside.emitted('update:sessionKind')).toEqual([['CHAT']])
  })
})
