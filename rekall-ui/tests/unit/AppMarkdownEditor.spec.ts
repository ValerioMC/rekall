import { describe, expect, it, vi } from 'vitest'
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

  it('turns a mermaid fence into a diagram container without reaching for a cdn', async () => {
    const wrapper = mount(AppMarkdownEditor, {
      props: { modelValue: '```mermaid\ngraph TD\n  A --> B\n```', readonly: true }
    })
    await flushPromises()
    await nextTick()

    expect(wrapper.find('.md-editor-mermaid').exists()).toBe(true)
    expect(wrapper.find('pre code').exists()).toBe(false)
    expect(document.getElementById('md-editor-mermaid')).toBeNull()
    expect(document.getElementById('md-editor-mermaid-m')).toBeNull()
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

  it('writes in one pane, drawn as the page, with no split preview beside it', async () => {
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: '# Titolo\n\nUn **punto**.' } })
    await flushPromises()

    expect(wrapper.find('.md-editor-preview-wrapper').exists()).toBe(false)
    const titles = wrapper.findAll('.md-editor-toolbar-item').map((item) => item.attributes('title'))
    expect(titles).not.toContain('preview')
    expect(wrapper.find('.cm-content .cm-rk-h1').text()).toBe('Titolo')
    expect(wrapper.find('.cm-content .cm-rk-strong').text()).toBe('punto')
  })

  it('adds a table row from the right-click menu and hands back the new markdown', async () => {
    const wrapper = mount(AppMarkdownEditor, {
      props: { modelValue: '| a | b |\n|---|---|\n| 1 | 2 |' },
      attachTo: document.body
    })
    await flushPromises()

    wrapper
      .find('.cm-rk-table td[data-column="0"]')
      .element.dispatchEvent(
        new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: 10, clientY: 10 })
      )
    await nextTick()
    const menu = document.querySelector('[data-testid="table-context-menu"]')
    expect(menu).not.toBeNull()
    const deleteColumn = document.querySelector<HTMLButtonElement>('[data-testid="table-menu-deleteColumn"]')
    expect(deleteColumn?.disabled).toBe(false)

    document.querySelector<HTMLButtonElement>('[data-testid="table-menu-rowBelow"]')?.click()
    await nextTick()

    expect(wrapper.emitted('update:modelValue')?.at(-1)).toEqual([
      '| a | b |\n| --- | --- |\n| 1 | 2 |\n|  |  |'
    ])
    expect(document.querySelector('[data-testid="table-context-menu"]')).toBeNull()
    wrapper.unmount()
  })

  it('refits a growing editor when its content resizes, as a diagram does once drawn', async () => {
    const observed: Element[] = []
    vi.stubGlobal(
      'ResizeObserver',
      class {
        observe(target: Element): void {
          observed.push(target)
        }
        disconnect(): void {}
      }
    )
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: 'Testo.', grow: true } })
    await flushPromises()

    expect(observed).toContain(wrapper.find('.cm-content').element)
    wrapper.unmount()
    vi.unstubAllGlobals()
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

  it('draws a [[Name]] as a chip for a component of the project and flags one it does not have', async () => {
    const component = {
      id: 'c-1', projectId: 'p-1', name: 'Auth Service', folder: 'services/auth', description: 'Issues tokens.',
      position: 0, createdAt: '', updatedAt: ''
    }
    const wrapper = mount(AppMarkdownEditor, {
      props: { modelValue: 'See [[auth service]] and [[Billing]].', readonly: true, components: [component] as never }
    })
    await flushPromises()
    await nextTick()

    const chips = wrapper.findAll('.rk-component-ref').map((chip) => chip.attributes('data-state'))
    expect(chips).toEqual(['found', 'missing'])

    await wrapper.setProps({ components: [{ ...component, name: 'Identity' }] as never })
    await flushPromises()
    expect(wrapper.findAll('.rk-component-ref').map((chip) => chip.attributes('data-state'))).toEqual(['missing', 'missing'])
  })

  it('leaves a [[Name]] plain when the text belongs to no project', async () => {
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: 'See [[Auth Service]].', readonly: true } })
    await flushPromises()
    await nextTick()

    expect(wrapper.find('.rk-component-ref').attributes('data-state')).toBe('plain')
  })
})
