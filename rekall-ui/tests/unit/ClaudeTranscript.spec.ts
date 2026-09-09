import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ClaudeTranscript from '@/components/claude/ClaudeTranscript.vue'
import type { ClaudeMessage, ClaudeMessageRole } from '@/model/claude'
import type { ClaudeMessageId, ClaudeSessionId } from '@/model/branded'

let seq = 0

function message(role: ClaudeMessageRole, over: Partial<ClaudeMessage> = {}): ClaudeMessage {
  seq += 1
  return {
    id: `m-${seq}` as ClaudeMessageId,
    sessionId: 's-1' as ClaudeSessionId,
    seq,
    role,
    content: '',
    toolName: null,
    meta: null,
    createdAt: '2026-09-09T10:00:00Z',
    ...over
  }
}

describe('ClaudeTranscript', () => {
  it('folds each tool result into the call it answers', () => {
    const messages: ClaudeMessage[] = [
      message('ASSISTANT', { content: 'Checking.' }),
      message('TOOL_USE', { toolName: 'Bash', content: '{"command":"a"}', meta: '{"toolUseId":"tu_1"}' }),
      message('TOOL_USE', { toolName: 'Bash', content: '{"command":"b"}', meta: '{"toolUseId":"tu_2"}' }),
      message('TOOL_RESULT', { content: 'out b', meta: '{"toolUseId":"tu_2"}' }),
      message('TOOL_RESULT', { content: 'out a', meta: '{"toolUseId":"tu_1"}' })
    ]
    const wrapper = mount(ClaudeTranscript, { props: { messages, working: false } })

    const summaries = wrapper
      .findAll('[data-testid="claude-tool-call"]')
      .map((call) => call.find('summary').text())
    expect(summaries).toEqual([
      expect.stringContaining('out a'),
      expect.stringContaining('out b')
    ])
  })

  it('pairs by order when no id is present', () => {
    const messages: ClaudeMessage[] = [
      message('TOOL_USE', { toolName: 'Read', content: '{"file_path":"/a"}' }),
      message('TOOL_RESULT', { content: 'file body' })
    ]
    const wrapper = mount(ClaudeTranscript, { props: { messages, working: false } })

    const summaries = wrapper
      .findAll('[data-testid="claude-tool-call"]')
      .map((call) => call.find('summary').text())
    expect(summaries).toEqual([expect.stringContaining('file body')])
  })
})
