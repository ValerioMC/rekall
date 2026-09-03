import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'

/**
 * The editor comes and goes with every pane swap, so whatever it hangs on `document` and
 * `window` has to go with it. A listener left behind is never a visible bug: it is the whole
 * editor DOM kept alive through its closure, a megabyte or more per swap, and a memory graph
 * that only ever climbs. Balanced add/remove calls are the observable side of that.
 */
type Registration = { target: string; type: string; listener: EventListenerOrEventListenerObject }

function watchListeners(): { added: Registration[]; removed: Registration[]; restore: () => void } {
  const added: Registration[] = []
  const removed: Registration[] = []
  const targets: Array<[string, EventTarget]> = [
    ['document', document],
    ['window', window]
  ]
  const spies = targets.map(([name, target]) => {
    const originalAdd = target.addEventListener.bind(target)
    const originalRemove = target.removeEventListener.bind(target)
    const add = vi.spyOn(target, 'addEventListener').mockImplementation((type, listener, options) => {
      if (listener) added.push({ target: name, type, listener })
      originalAdd(type, listener, options)
    })
    const remove = vi.spyOn(target, 'removeEventListener').mockImplementation((type, listener, options) => {
      if (listener) removed.push({ target: name, type, listener })
      originalRemove(type, listener, options)
    })
    return [add, remove]
  })
  return { added, removed, restore: () => spies.flat().forEach((spy) => spy.mockRestore()) }
}

describe('AppMarkdownEditor lifecycle', () => {
  let watched: ReturnType<typeof watchListeners>

  beforeEach(() => {
    watched = watchListeners()
  })

  afterEach(() => {
    watched.restore()
  })

  it('removes every document and window listener it added once unmounted', async () => {
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: '# prima' } })
    await flushPromises()
    expect(wrapper.find('.cm-content[contenteditable]').exists()).toBe(true)

    wrapper.unmount()
    await flushPromises()

    const leftBehind = watched.added.filter(
      (registration) =>
        !watched.removed.some(
          (removal) =>
            removal.target === registration.target &&
            removal.type === registration.type &&
            removal.listener === registration.listener
        )
    )
    expect(leftBehind.map((registration) => `${registration.target}:${registration.type}`)).toEqual([])
  })

  it('gives each instance its own id, so two on screen do not collide', async () => {
    // Two in one application, which is what the steps pane puts on screen with two steps open.
    const TwoPreviews = defineComponent({
      render: () =>
        h('div', [
          h(AppMarkdownEditor, { modelValue: 'uno', readonly: true }),
          h(AppMarkdownEditor, { modelValue: 'due', readonly: true })
        ])
    })
    const wrapper = mount(TwoPreviews)
    await flushPromises()

    const ids = wrapper.findAll('.md-editor').map((editor) => editor.attributes('id'))
    expect(ids).toHaveLength(2)
    expect(ids[0]).toBeTruthy()
    expect(ids[0]).not.toBe(ids[1])

    wrapper.unmount()
  })
})
