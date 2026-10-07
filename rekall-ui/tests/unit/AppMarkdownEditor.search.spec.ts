import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { nextTick } from 'vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'

const TEXT = '# Export\n\nThe export formats. Export to csv.\n\n- plain list'

let wrapper: VueWrapper | null = null

async function mountReadView(props: Record<string, unknown> = {}): Promise<VueWrapper> {
  wrapper = mount(AppMarkdownEditor, {
    props: { modelValue: TEXT, readonly: true, ...props },
    attachTo: document.body
  })
  await flushPromises()
  await nextTick()
  return wrapper
}

function pressFind(init: KeyboardEventInit = { metaKey: true }): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { code: 'KeyF', cancelable: true, ...init })
  window.dispatchEvent(event)
  return event
}

async function typeQuery(view: VueWrapper, text: string): Promise<void> {
  await view.get('[data-testid="read-search-input"]').setValue(text)
  await nextTick()
}

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('AppMarkdownEditor find in read mode', () => {
  it('opens the find bar on cmd+F and takes the shortcut from the browser', async () => {
    const view = await mountReadView()

    const event = pressFind()
    await nextTick()

    expect(event.defaultPrevented).toBe(true)
    expect(view.find('[data-testid="read-search"]').exists()).toBe(true)
  })

  it('opens on ctrl+F too', async () => {
    const view = await mountReadView()

    pressFind({ ctrlKey: true })
    await nextTick()

    expect(view.find('[data-testid="read-search"]').exists()).toBe(true)
  })

  it('counts the matches, ignoring case', async () => {
    const view = await mountReadView()
    pressFind()
    await nextTick()

    await typeQuery(view, 'export')

    expect(view.get('[data-testid="read-search-count"]').text()).toBe('1/3')
  })

  it('steps through the matches with Enter and back with Shift+Enter, wrapping around', async () => {
    const view = await mountReadView()
    pressFind()
    await nextTick()
    await typeQuery(view, 'export')
    const input = view.get('[data-testid="read-search-input"]')

    await input.trigger('keydown', { key: 'Enter' })
    expect(view.get('[data-testid="read-search-count"]').text()).toBe('2/3')

    await input.trigger('keydown', { key: 'Enter', shiftKey: true })
    await input.trigger('keydown', { key: 'Enter', shiftKey: true })
    expect(view.get('[data-testid="read-search-count"]').text()).toBe('3/3')
  })

  it('keeps the position after the bar repaints its own counter', async () => {
    const view = await mountReadView()
    pressFind()
    await nextTick()
    await typeQuery(view, 'export')

    await view.get('[data-testid="read-search-next"]').trigger('click')
    await flushPromises()
    await new Promise((resolve) => setTimeout(resolve))

    expect(view.get('[data-testid="read-search-count"]').text()).toBe('2/3')
  })

  it('shows 0/0 for a query nothing matches', async () => {
    const view = await mountReadView()
    pressFind()
    await nextTick()

    await typeQuery(view, 'zzz')

    expect(view.get('[data-testid="read-search-count"]').text()).toBe('0/0')
  })

  it('closes on Escape and forgets the query', async () => {
    const view = await mountReadView()
    pressFind()
    await nextTick()
    await typeQuery(view, 'export')

    await view.get('[data-testid="read-search-input"]').trigger('keydown', { key: 'Escape' })
    expect(view.find('[data-testid="read-search"]').exists()).toBe(false)

    pressFind()
    await nextTick()
    expect((view.get('[data-testid="read-search-input"]').element as HTMLInputElement).value).toBe('')
  })

  it('searches again when the text changes while the bar is open', async () => {
    const view = await mountReadView()
    pressFind()
    await nextTick()
    await typeQuery(view, 'export')

    await view.setProps({ modelValue: 'export once' })
    await vi.waitFor(() => expect(view.get('.md-editor-preview').text()).toBe('export once'))
    await flushPromises()

    expect(view.get('[data-testid="read-search-count"]').text()).toBe('1/1')
  })

  it('leaves cmd+F alone in a card-sized read view', async () => {
    await mountReadView({ compact: true })

    const event = pressFind()

    expect(event.defaultPrevented).toBe(false)
  })

  it('leaves cmd+F to the editor in write mode', async () => {
    await mountReadView({ readonly: false })

    const event = pressFind()

    expect(event.defaultPrevented).toBe(false)
  })

  it('gives the shortcut to the read view mounted last', async () => {
    const below = await mountReadView()
    const above = mount(AppMarkdownEditor, { props: { modelValue: TEXT, readonly: true }, attachTo: document.body })
    await flushPromises()

    pressFind()
    await nextTick()

    expect(above.find('[data-testid="read-search"]').exists()).toBe(true)
    expect(below.find('[data-testid="read-search"]').exists()).toBe(false)
    above.unmount()
  })
})
