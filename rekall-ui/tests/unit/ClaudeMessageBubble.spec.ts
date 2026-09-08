import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ClaudeMessageBubble from '@/components/claude/ClaudeMessageBubble.vue'
import type { ClaudeMessage, ClaudeMessageRole } from '@/model/claude'
import type { ClaudeMessageId, ClaudeSessionId } from '@/model/branded'

function message(role: ClaudeMessageRole, over: Partial<ClaudeMessage> = {}): ClaudeMessage {
  return {
    id: 'm-1' as ClaudeMessageId,
    sessionId: 's-1' as ClaudeSessionId,
    seq: 0,
    role,
    content: '',
    toolName: null,
    meta: null,
    createdAt: '2026-09-08T10:00:00Z',
    ...over
  }
}

describe('ClaudeMessageBubble', () => {
  it('renders a user prompt with its own label', () => {
    const wrapper = mount(ClaudeMessageBubble, {
      props: { message: message('USER', { content: 'what changed?' }) }
    })
    expect(wrapper.get('[data-testid="claude-msg-user"]').text()).toContain('what changed?')
    expect(wrapper.text()).toContain('you')
  })

  it('splits an assistant message into text and fenced code', () => {
    const wrapper = mount(ClaudeMessageBubble, {
      props: {
        message: message('ASSISTANT', {
          content: 'Here it is:\n```ts\nconst x = 1\n```\nDone.'
        })
      }
    })
    const bubble = wrapper.get('[data-testid="claude-msg-assistant"]')
    expect(bubble.findAll('pre')).toHaveLength(1)
    expect(bubble.get('pre').text()).toBe('const x = 1')
    expect(bubble.text()).toContain('Here it is:')
    expect(bubble.text()).toContain('Done.')
  })

  it('hides a tool call input until it is expanded', async () => {
    const wrapper = mount(ClaudeMessageBubble, {
      props: {
        message: message('TOOL_USE', { toolName: 'Read', content: '{"file_path":"/a/b.txt"}' })
      }
    })
    expect(wrapper.text()).toContain('Read')
    expect(wrapper.find('pre').exists()).toBe(false)
    await wrapper.get('button').trigger('click')
    expect(wrapper.get('pre').text()).toContain('/a/b.txt')
  })

  it('turns result meta into a stat line', () => {
    const wrapper = mount(ClaudeMessageBubble, {
      props: {
        message: message('RESULT', {
          meta: JSON.stringify({ numTurns: 3, durationMs: 2500, costUsd: 0.012 })
        })
      }
    })
    const text = wrapper.get('[data-testid="claude-msg-result"]').text()
    expect(text).toContain('3 turns')
    expect(text).toContain('2.5s')
    expect(text).toContain('$0.012')
  })

  it('renders a system note centred and quiet', () => {
    const wrapper = mount(ClaudeMessageBubble, {
      props: { message: message('SYSTEM', { content: 'Session opened.' }) }
    })
    expect(wrapper.get('[data-testid="claude-msg-system"]').text()).toBe('Session opened.')
  })
})
