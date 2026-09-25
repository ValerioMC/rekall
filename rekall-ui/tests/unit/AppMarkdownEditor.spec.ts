import { describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'

/**
 * md-editor-v3 fetches highlight.js from unpkg unless it is handed a local instance, and it
 * does so lazily, on the first fenced block rather than on mount. That makes the regression
 * invisible in development on a working connection and total on a train, so the assertion is
 * on the injection itself: the library tags what it appends with a known id.
 */
const HLJS_SCRIPT_ID = 'md-editor-hljs'
const HLJS_STYLE_ID = 'md-editor-hlCss'

const WITH_CODE = '# Contesto\n\n```java\nrecord Task(String name) {}\n```\n'

describe('AppMarkdownEditor', () => {
  it('renders markdown as html when read only', async () => {
    const wrapper = mount(AppMarkdownEditor, {
      props: { modelValue: '# Contesto\n\nUsa `kubectl` qui.', readonly: true }
    })
    await flushPromises()
    await nextTick()

    expect(wrapper.html()).toContain('Contesto')
    expect(wrapper.find('h1').exists()).toBe(true)
    expect(wrapper.find('code').exists()).toBe(true)
  })

  it('offers the reading controls on a pane-sized page', async () => {
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: 'Una riga.', readonly: true } })
    await flushPromises()

    expect(wrapper.find('[data-testid="read-settings-toggle"]').exists()).toBe(true)
    expect(wrapper.classes()).not.toContain('rekall-md--compact')
  })

  it('drops the reading controls and the column width when compact, inside a card', async () => {
    const wrapper = mount(AppMarkdownEditor, {
      props: { modelValue: 'Una riga.', readonly: true, compact: true }
    })
    await flushPromises()

    expect(wrapper.find('[data-testid="read-settings-toggle"]').exists()).toBe(false)
    expect(wrapper.classes()).toContain('rekall-md--compact')
    expect(wrapper.attributes('style') ?? '').not.toContain('--rk-read-width')
    expect(wrapper.attributes('data-align')).toBeUndefined()
  })

  it('renders a table, which the previous hand-written renderer could not', async () => {
    const wrapper = mount(AppMarkdownEditor, {
      props: { modelValue: '| Step | Esito |\n|------|-------|\n| 1 | ok |', readonly: true }
    })
    await flushPromises()
    await nextTick()

    expect(wrapper.find('table').exists()).toBe(true)
    expect(wrapper.findAll('td').length).toBe(2)
  })

  it('highlights code without reaching for a cdn', async () => {
    const wrapper = mount(AppMarkdownEditor, {
      props: { modelValue: WITH_CODE, readonly: true }
    })
    await flushPromises()
    await nextTick()

    expect(document.getElementById(HLJS_SCRIPT_ID)).toBeNull()
    expect(document.getElementById(HLJS_STYLE_ID)).toBeNull()

    const external = [...document.querySelectorAll('script[src], link[href]')].map(
      (node) => node.getAttribute('src') ?? node.getAttribute('href') ?? ''
    )
    expect(external.filter((url) => /^https?:/.test(url))).toEqual([])

    // The local instance is in use rather than merely present: hljs marks up what it parses.
    expect(wrapper.find('.hljs-keyword, .hljs-string, .hljs-title, .hljs-type').exists()).toBe(true)
  })

  it('offers the formatting toolbar over an editable surface', async () => {
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: '# prima' } })
    await flushPromises()

    const buttons = wrapper.findAll('.md-editor-toolbar-item').map((item) => item.attributes('title'))
    expect(buttons).toEqual(
      expect.arrayContaining([
        'bold',
        'italic',
        'title',
        'unordered list',
        'ordered list',
        'block-level code',
        'link',
        'table'
      ])
    )
    expect(wrapper.find('.cm-content[contenteditable]').exists()).toBe(true)
  })

  it('puts the path picker in the toolbar, next to link', async () => {
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: '', pathRoot: '/work/app' } })
    await flushPromises()

    const titles = wrapper.findAll('.md-editor-toolbar-item').map((item) => item.attributes('title'))
    expect(titles[titles.indexOf('link') + 1]).toBe('Insert a path  ⌘⇧P')
    const trigger = wrapper.find('[data-testid="path-picker-trigger"]')
    expect(trigger.attributes('data-path-picker-trigger')).toBeDefined()
    expect(trigger.attributes('aria-expanded')).toBe('false')
  })

  /**
   * Each of these draws its implementation from unpkg on first click. They are left out of the
   * toolbar rather than merely unused, so that a button cannot put the application in a state
   * it can only leave with a network round trip.
   */
  it('offers no control that would need a library fetched at runtime', async () => {
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: '# prima' } })
    await flushPromises()

    const buttons = wrapper.findAll('.md-editor-toolbar-item').map((item) => item.attributes('title'))
    expect(buttons).not.toEqual(
      expect.arrayContaining(['fullscreen', 'prettier', 'katex', 'mermaid'])
    )
    expect(document.getElementById(HLJS_SCRIPT_ID)).toBeNull()
  })
})
