import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ClaudeToolCall from '@/components/claude/ClaudeToolCall.vue'
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
    createdAt: '2026-09-09T10:00:00Z',
    ...over
  }
}

describe('ClaudeToolCall', () => {
  it('reads the call on one line without being opened', () => {
    const wrapper = mount(ClaudeToolCall, {
      props: {
        call: message('TOOL_USE', { toolName: 'Bash', content: '{"command":"git status --porcelain"}' }),
        result: null
      }
    })
    const summary = wrapper.get('summary')
    expect(summary.text()).toContain('Bash')
    expect(summary.text()).toContain('git status --porcelain')
  })

  it('shows a preview of what the result returned', () => {
    const wrapper = mount(ClaudeToolCall, {
      props: {
        call: message('TOOL_USE', { toolName: 'Read', content: '{"file_path":"/a/b.ts"}' }),
        result: message('TOOL_RESULT', { id: 'm-2' as ClaudeMessageId, content: 'line one\nline two\nline three' })
      }
    })
    expect(wrapper.get('summary').text()).toContain('line one')
    expect(wrapper.get('summary').text()).toContain('+2')
  })

  it('marks an errored result and shows its body in danger tone', () => {
    const wrapper = mount(ClaudeToolCall, {
      props: {
        call: message('TOOL_USE', { toolName: 'Bash', content: '{"command":"false"}' }),
        result: message('TOOL_RESULT', {
          id: 'm-2' as ClaudeMessageId,
          content: 'boom',
          meta: '{"toolUseId":"tu_1","error":true}'
        })
      }
    })
    expect(wrapper.get('summary').text()).toContain('error')
    expect(wrapper.get('summary').find('.text-danger').exists()).toBe(true)
  })

  it('reveals the full input and result once opened', () => {
    const wrapper = mount(ClaudeToolCall, {
      props: {
        call: message('TOOL_USE', { toolName: 'Edit', content: '{"file_path":"/a/b.ts","old_string":"x"}' }),
        result: message('TOOL_RESULT', { id: 'm-2' as ClaudeMessageId, content: 'applied' })
      }
    })
    const bodies = wrapper.findAll('pre').map((pre) => pre.text())
    expect(bodies).toHaveLength(2)
    expect(bodies.join('\n')).toContain('old_string')
    expect(bodies.join('\n')).toContain('applied')
  })

  it('still renders a result with no matching call', () => {
    const wrapper = mount(ClaudeToolCall, {
      props: {
        call: null,
        result: message('TOOL_RESULT', { content: 'orphan output' })
      }
    })
    expect(wrapper.get('summary').text()).toContain('tool')
    expect(wrapper.get('pre').text()).toContain('orphan output')
  })
})
