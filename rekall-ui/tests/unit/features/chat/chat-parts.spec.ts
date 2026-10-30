import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ChatNotice from '@/features/chat/components/ChatNotice.vue'
import ChatThinking from '@/features/chat/components/ChatThinking.vue'
import ChatTurnFooter from '@/features/chat/components/ChatTurnFooter.vue'

const USAGE = { inputTokens: 1200, cacheCreationInputTokens: 2000, cacheReadInputTokens: 38000, outputTokens: 640 }

describe('ChatTurnFooter', () => {
  it('prints what the turn cost on one line', () => {
    const footer = mount(ChatTurnFooter, { props: { usage: USAGE, durationMs: 12400, outcome: 'completed' } })

    expect(footer.text()).toBe('in 1.2k · cache read 38k · cache write 2k · out 640 · 12.4 s')
  })

  it('says when the turn was stopped or failed', () => {
    const stopped = mount(ChatTurnFooter, { props: { usage: USAGE, durationMs: 1000, outcome: 'interrupted' } })
    const failed = mount(ChatTurnFooter, { props: { usage: USAGE, durationMs: 1000, outcome: 'failed' } })

    expect(stopped.text()).toContain('stopped')
    expect(failed.text()).toContain('failed')
  })
})

describe('ChatThinking', () => {
  const text = `${'a thought '.repeat(20)}`

  it('is folded to a line of its first 80 characters, and never opens by itself', async () => {
    const thinking = mount(ChatThinking, { props: { text, complete: false } })

    expect(thinking.get('button').attributes('aria-expanded')).toBe('false')
    expect(thinking.text()).toContain('Thinking')
    expect(thinking.get('span.truncate').text()).toHaveLength(81)
    expect(thinking.get('span.truncate').text().endsWith('…')).toBe(true)

    await thinking.setProps({ text: `${text} and more`, complete: true })
    expect(thinking.get('button').attributes('aria-expanded')).toBe('false')
  })

  it('opens to the whole text', async () => {
    const thinking = mount(ChatThinking, { props: { text: 'first\nsecond', complete: true } })

    await thinking.get('button').trigger('click')

    expect(thinking.get('p').text()).toBe('first\nsecond')
  })
})

describe('ChatNotice', () => {
  it('is a status line carrying what happened', () => {
    const notice = mount(ChatNotice, { props: { kind: 'compaction', text: 'Context compacted' } })

    expect(notice.attributes('role')).toBe('status')
    expect(notice.text()).toBe('Context compacted')
  })
})
