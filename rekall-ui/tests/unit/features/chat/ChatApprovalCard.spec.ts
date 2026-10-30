import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ChatApprovalCard from '@/features/chat/components/ChatApprovalCard.vue'
import type { PendingPermission } from '@/features/chat/model/timeline'

const BASH: PendingPermission = {
  requestId: 'req-1',
  toolUseId: 'u1',
  name: 'Bash',
  input: { tool: 'bash', command: 'rm -rf target', description: null }
}

function mountCard(over: { request?: PendingPermission; busy?: boolean; error?: string | null } = {}) {
  return mount(ChatApprovalCard, {
    props: { request: BASH, busy: false, error: null, ...over },
    attachTo: document.body
  })
}

describe('ChatApprovalCard', () => {
  it('says in words what is asked and shows the input', () => {
    const card = mountCard()

    expect(card.get('[data-testid=chat-approval-sentence]').text()).toBe('Claude wants to run a shell command')
    expect(card.text()).toContain('rm -rf target')
    expect(card.get('[role=group]').attributes('aria-label')).toBe('Permission needed')
    card.unmount()
  })

  it('takes focus on itself, not on a key, when it appears', () => {
    const card = mountCard()

    expect(document.activeElement).toBe(card.get('[data-testid=chat-approval]').element)
    card.unmount()
  })

  it('emits allow and deny from its keys', async () => {
    const card = mountCard()

    await card.get('[data-testid=chat-allow]').trigger('click')
    await card.get('[data-testid=chat-deny]').trigger('click')

    expect(card.emitted('allow')).toHaveLength(1)
    expect(card.emitted('deny')).toHaveLength(1)
    card.unmount()
  })

  it('answers to Y and N while focus is inside it', async () => {
    const card = mountCard()

    await card.get('[data-testid=chat-approval]').trigger('keydown', { key: 'y' })
    await card.get('[data-testid=chat-approval]').trigger('keydown', { key: 'N' })

    expect(card.emitted('allow')).toHaveLength(1)
    expect(card.emitted('deny')).toHaveLength(1)
    card.unmount()
  })

  it('does not answer a second time while the first answer is on its way', async () => {
    const card = mountCard({ busy: true })

    await card.get('[data-testid=chat-approval]').trigger('keydown', { key: 'y' })

    expect(card.emitted('allow')).toBeUndefined()
    expect(card.get('[data-testid=chat-allow]').attributes('disabled')).toBeDefined()
    expect(card.get('[data-testid=chat-deny]').attributes('disabled')).toBeDefined()
    card.unmount()
  })

  it('shows a failed answer inline as an alert', () => {
    const card = mountCard({ error: 'Could not reach Rekall' })

    expect(card.get('[role=alert]').text()).toBe('Could not reach Rekall')
    card.unmount()
  })
})
