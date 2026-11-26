import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import SharedSessionSection from '@/features/settings/components/SharedSessionSection.vue'
import QueueSettingsAside from '@/features/queue/components/QueueSettingsAside.vue'
import { preferredSessionKind } from '@/common/config/claude-launch'
import { installStorage } from '../../support/memoryStorage'

vi.mock('@/features/claude/api/claude.api', () => ({
  fetchClaudeUsage: vi.fn().mockResolvedValue({ status: 'UNAVAILABLE', limits: [], fetchedAt: '' })
}))

describe('Settings > Both providers > Session view', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
  })

  it('offers the terminal and the chat, marks the chat experimental, and keeps the choice', async () => {
    const section = mount(SharedSessionSection)
    const view = section.get('[data-testid=session-view]')
    const terminal = view.get('[data-testid=segment-terminal]')
    const chat = view.get('[data-testid=segment-chat]')
    expect(terminal.attributes('aria-pressed')).toBe('true')
    expect(chat.text()).toBe('Chat (experimental)')

    await chat.trigger('click')

    expect(chat.attributes('aria-pressed')).toBe('true')
    expect(terminal.attributes('aria-pressed')).toBe('false')
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
