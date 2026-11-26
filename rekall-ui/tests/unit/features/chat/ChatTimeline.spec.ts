import { describe, expect, it } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import { nextTick } from 'vue'
import ChatTimeline from '@/features/chat/components/ChatTimeline.vue'
import type { TimelineItem, ToolItem } from '@/features/chat/model/timeline'
import { MarkdownStub, toolItem, userItem } from './support'

function items(count: number, from = 1): TimelineItem[] {
  return Array.from({ length: count }, (_, index) => userItem(from + index))
}

function mountTimeline(initial: readonly TimelineItem[], followingNext = false): VueWrapper {
  return mount(ChatTimeline, {
    props: { items: initial, followingNext },
    attachTo: document.body,
    global: { stubs: { AppMarkdownEditor: MarkdownStub } }
  })
}

function scrollerOf(timeline: VueWrapper): HTMLElement {
  const element = timeline.get('[data-testid=chat-timeline]').element as HTMLElement
  let top = 0
  Object.defineProperty(element, 'scrollHeight', { configurable: true, get: () => 1000 })
  Object.defineProperty(element, 'clientHeight', { configurable: true, get: () => 400 })
  Object.defineProperty(element, 'scrollTop', { configurable: true, get: () => top, set: (value: number) => (top = value) })
  return element
}

async function scrollTo(element: HTMLElement, top: number): Promise<void> {
  element.scrollTop = top
  element.dispatchEvent(new Event('scroll'))
  await nextTick()
}

describe('ChatTimeline', () => {
  it('is a log the keyboard can scroll', () => {
    const timeline = mountTimeline(items(2))
    const log = timeline.get('[role=log]')

    expect(log.attributes('aria-label')).toBe('Conversation')
    expect(log.attributes('aria-live')).toBe('off')
    expect(log.attributes('tabindex')).toBe('0')
    timeline.unmount()
  })

  it('follows new items while the reader is at the bottom, and offers no jump', async () => {
    const timeline = mountTimeline(items(3))
    const element = scrollerOf(timeline)

    await timeline.setProps({ items: items(4) })
    await nextTick()

    expect(element.scrollTop).toBe(1000)
    expect(timeline.find('[data-testid=chat-jump-to-latest]').exists()).toBe(false)
    timeline.unmount()
  })

  it('lets go when the reader scrolls up, counts what arrives, and jumps back on request', async () => {
    const timeline = mountTimeline(items(3))
    const element = scrollerOf(timeline)

    await scrollTo(element, 100)
    expect(timeline.get('[data-testid=chat-jump-to-latest]').text()).toBe('Jump to latest')

    await timeline.setProps({ items: items(5) })
    await nextTick()
    expect(element.scrollTop).toBe(100)
    expect(timeline.get('[data-testid=chat-jump-to-latest]').text()).toBe('Jump to latest · 2 new')

    await timeline.get('[data-testid=chat-jump-to-latest]').trigger('click')
    expect(element.scrollTop).toBe(1000)
    expect(timeline.find('[data-testid=chat-jump-to-latest]').exists()).toBe(false)
    timeline.unmount()
  })

  it('sticks again when the reader scrolls back to the bottom', async () => {
    const timeline = mountTimeline(items(3))
    const element = scrollerOf(timeline)

    await scrollTo(element, 100)
    await scrollTo(element, 570)

    expect(timeline.find('[data-testid=chat-jump-to-latest]').exists()).toBe(false)
    timeline.unmount()
  })

  it('mounts the last 150 items, and brings 150 more back on request', async () => {
    const timeline = mountTimeline(items(400))
    expect(timeline.findAll('li[class=""], li').filter((row) => row.text().startsWith('message'))).toHaveLength(150)
    expect(timeline.get('[data-testid=chat-show-earlier]').text()).toBe('Show 150 earlier items')
    expect(timeline.text()).toContain('message 400')
    expect(timeline.text()).not.toContain('message 250 ')

    await timeline.get('[data-testid=chat-show-earlier]').trigger('click')
    await nextTick()

    expect(timeline.findAll('li').filter((row) => row.text().startsWith('message'))).toHaveLength(300)
    expect(timeline.get('[data-testid=chat-show-earlier]').text()).toBe('Show 100 earlier items')
    timeline.unmount()
  })

  it('shows the end of the session, waiting for the next one when it is on its way', () => {
    const ended: TimelineItem = { kind: 'ended', key: 'ended-1', exitCode: 0, detail: 'finished' }

    const waiting = mountTimeline([ended], true)
    const done = mountTimeline([ended], false)

    expect(waiting.get('[data-testid=chat-ended-following]').text()).toContain('Opening the next session')
    expect(done.find('[data-testid=chat-ended-following]').exists()).toBe(false)
    expect(done.text()).toContain('Start a new one with Run here')
    waiting.unmount()
    done.unmount()
  })

  function agentCall(over: Partial<ToolItem> = {}): ToolItem {
    return toolItem({
      key: 'tool-agent-1',
      toolUseId: 'agent-1',
      input: { tool: 'agent', description: 'Find the parser', subagentType: null, prompt: 'go' },
      ...over
    })
  }

  function innerRead(index: number): ToolItem {
    return toolItem({
      key: `tool-inner-${index}`,
      toolUseId: `inner-${index}`,
      input: { tool: 'read', filePath: `src/file-${index}.rs` },
      state: 'done',
      parentToolUseId: 'agent-1'
    })
  }

  it('draws a subagent\'s items inside its Agent card, not in the conversation', () => {
    const timeline = mountTimeline([agentCall(), innerRead(1), innerRead(2), userItem(1)])

    const inner = timeline.get('[data-testid=agent-inner]')
    expect(inner.findAll('[data-testid=tool-call-card]')).toHaveLength(2)
    expect(inner.text()).toContain('src/file-1.rs')
    expect(timeline.findAll('[data-testid=tool-call-card]')).toHaveLength(3)
    expect(timeline.get('ol').element.children).toHaveLength(2)
    timeline.unmount()
  })

  it('keeps an item whose parent is not an Agent call in the conversation', () => {
    const orphan = toolItem({ key: 'tool-x', toolUseId: 'x', input: { tool: 'read', filePath: 'a.rs' }, parentToolUseId: 'ghost' })

    const timeline = mountTimeline([orphan])

    expect(timeline.find('[data-testid=agent-inner]').exists()).toBe(false)
    expect(timeline.findAll('[data-testid=tool-call-card]')).toHaveLength(1)
    timeline.unmount()
  })

  it('counts the window over the conversation, never over what a subagent did', () => {
    const subagent = Array.from({ length: 200 }, (_, index) => innerRead(index))

    const timeline = mountTimeline([agentCall(), ...subagent, ...items(100)])

    expect(timeline.find('[data-testid=chat-show-earlier]').exists()).toBe(false)
    expect(timeline.findAll('[data-testid=agent-inner] [data-testid=tool-call-card]')).toHaveLength(200)
    timeline.unmount()
  })

  it('tells a command by its description alone, and leaves out one that has none', () => {
    const described = toolItem({
      input: { tool: 'bash', command: 'cargo test --workspace', description: 'Run every test' },
      state: 'done',
      result: { isError: false, excerpt: '12 passed', truncated: false, summary: 'exit code 0' }
    })
    const bare = toolItem({ key: 'tool-u2', toolUseId: 'u2', input: { tool: 'bash', command: 'pwd', description: null } })

    const timeline = mountTimeline([described, bare])

    expect(timeline.get('[data-testid=chat-command-line]').text()).toBe('DoneRun every test')
    expect(timeline.findAll('li')).toHaveLength(1)
    expect(timeline.text()).not.toContain('cargo test')
    expect(timeline.text()).not.toContain('12 passed')
    expect(timeline.find('button').exists()).toBe(false)
    timeline.unmount()
  })

  it('shows the session row and the output of a command in place', () => {
    const timeline = mountTimeline([
      { kind: 'session', key: 'session-1', model: 'claude-opus', cwd: '/code/vega', permissionMode: 'default', version: '2.1.331' },
      { kind: 'command-output', key: 'command-2', command: '/context', text: 'Context Usage\n10%' }
    ])

    expect(timeline.get('[data-testid=chat-session-start]').text()).toContain('/code/vega')
    expect(timeline.get('[data-testid=chat-command-output]').text()).toContain('/context')
    timeline.unmount()
  })
})
