import MarkdownIt from 'markdown-it'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { markMermaidFences, sourceOfMermaidFrame } from '@/common/markdown/mermaid-fence'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { showStaticDiagram } from '@/features/diagram/lib/static-diagram'

vi.mock('@/features/diagram/lib/static-diagram', () => ({ showStaticDiagram: vi.fn() }))

const PAGE = 'Intro\n\n```mermaid\nsequenceDiagram\n  A->>B: "hi" <ok>\n```\n\n```java\nclass A {}\n```\n'

afterEach(() => vi.mocked(showStaticDiagram).mockReset())

describe('markMermaidFences', () => {
  function render(markdown: string): string {
    const md = new MarkdownIt({ html: true })
    markMermaidFences(md as never)
    return md.render(markdown)
  }

  it('leaves an empty frame carrying the source in place of a mermaid fence', () => {
    const page = document.createElement('div')
    page.innerHTML = render(PAGE)

    const frame = page.querySelector<HTMLElement>('.rk-mermaid-fence')
    expect(frame?.textContent).toBe('')
    expect(sourceOfMermaidFrame(frame as HTMLElement)).toBe('sequenceDiagram\n  A->>B: "hi" <ok>')
  })

  it('leaves the fences of other languages as code', () => {
    const html = render(PAGE)

    expect(html).toContain('language-java')
    expect(html.match(/rk-mermaid-fence/g)).toHaveLength(1)
  })
})

describe('a mermaid fence in a read view', () => {
  it('is drawn by the console from its source', async () => {
    vi.mocked(showStaticDiagram).mockImplementation(async (host) => {
      host.textContent = 'drawn'
    })
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: PAGE, readonly: true }, attachTo: document.body })
    await flushPromises()
    await new Promise((resolve) => setTimeout(resolve, 10))

    const frame = wrapper.find('.rk-mermaid-fence')
    expect(frame.text()).toBe('drawn')
    expect(vi.mocked(showStaticDiagram).mock.calls[0]?.[1]).toBe('sequenceDiagram\n  A->>B: "hi" <ok>')
    wrapper.unmount()
  })

  it('says why a diagram could not be drawn', async () => {
    vi.mocked(showStaticDiagram).mockRejectedValue(new Error('Parse error on line 2'))
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: PAGE, readonly: true }, attachTo: document.body })
    await flushPromises()
    await new Promise((resolve) => setTimeout(resolve, 10))

    const frame = wrapper.find('.rk-mermaid-fence')
    expect(frame.classes()).toContain('rk-mermaid-fence--failed')
    expect(frame.text()).toContain('Parse error on line 2')
    wrapper.unmount()
  })

  it('takes the drawing down when the view goes', async () => {
    vi.mocked(showStaticDiagram).mockResolvedValue(undefined)
    const wrapper = mount(AppMarkdownEditor, { props: { modelValue: PAGE, readonly: true }, attachTo: document.body })
    await flushPromises()
    await new Promise((resolve) => setTimeout(resolve, 10))
    const signal = vi.mocked(showStaticDiagram).mock.calls[0]?.[2] as AbortSignal

    wrapper.unmount()

    expect(signal.aborted).toBe(true)
  })
})
