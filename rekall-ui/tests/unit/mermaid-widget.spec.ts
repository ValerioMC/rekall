import { afterEach, describe, expect, it, vi } from 'vitest'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { livePreview } from '@/common/live-preview/live-preview'
import { showStaticDiagram } from '@/features/diagram/lib/static-diagram'

vi.mock('@/features/diagram/lib/static-diagram', () => ({ showStaticDiagram: vi.fn() }))

const FENCE = 'Intro\n\n```mermaid\nsequenceDiagram\n  A->>B: hi\n```\n\nOutro'

const views: EditorView[] = []

function editorOn(doc: string): EditorView {
  const view = new EditorView({
    parent: document.body.appendChild(document.createElement('div')),
    state: EditorState.create({ doc, selection: { anchor: 0 }, extensions: [markdown({ base: markdownLanguage }), livePreview()] })
  })
  views.push(view)
  return view
}

async function settled(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 10))
}

afterEach(() => {
  views.splice(0).forEach((view) => view.destroy())
  document.body.innerHTML = ''
  vi.mocked(showStaticDiagram).mockReset()
})

describe('the mermaid widget', () => {
  it('hands the fence source to the console drawing, into its own canvas', async () => {
    vi.mocked(showStaticDiagram).mockImplementation(async (host) => {
      host.textContent = 'drawn'
    })
    const view = editorOn(FENCE)
    await settled()

    expect(view.contentDOM.querySelector('.cm-rk-mermaid-canvas')?.textContent).toBe('drawn')
    expect(vi.mocked(showStaticDiagram).mock.calls[0]?.[1]).toBe('sequenceDiagram\n  A->>B: hi')
  })

  it('says why a diagram could not be drawn', async () => {
    vi.mocked(showStaticDiagram).mockRejectedValue(new Error('Parse error on line 2'))
    const view = editorOn(FENCE)
    await settled()

    const frame = view.contentDOM.querySelector('.cm-rk-mermaid')
    expect(frame?.classList.contains('cm-rk-mermaid--failed')).toBe(true)
    expect(frame?.textContent).toContain('Parse error on line 2')
  })

  it('takes the drawing down when the widget goes away', async () => {
    vi.mocked(showStaticDiagram).mockResolvedValue(undefined)
    const view = editorOn(FENCE)
    await settled()
    const signal = vi.mocked(showStaticDiagram).mock.calls[0]?.[2] as AbortSignal

    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: 'gone' } })

    expect(signal.aborted).toBe(true)
  })
})
