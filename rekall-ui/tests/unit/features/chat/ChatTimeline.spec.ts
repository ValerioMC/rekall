import { describe, expect, it } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import { nextTick } from 'vue'
import ChatTimeline from '@/features/chat/components/ChatTimeline.vue'
import type { TimelineItem } from '@/features/chat/model/timeline'
import { MarkdownStub, userItem } from './support'

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
})
