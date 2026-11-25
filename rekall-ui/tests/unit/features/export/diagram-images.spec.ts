import { describe, expect, it, vi } from 'vitest'
import { drawDiagramImages, mermaidSourcesOf, type DiagramRenderer } from '@/features/export/lib/diagram-images'
import type { DiagramProgress } from '@/features/export/model/diagram-image'

const FLOW = 'flowchart LR\n  A --> B'
const SEQUENCE = 'sequenceDiagram\n  A->>B: hi'

function fence(source: string): string {
  return `\`\`\`mermaid\n${source}\n\`\`\``
}

describe('mermaidSourcesOf', () => {
  it('returns the source of every mermaid fence, trimmed, in order', () => {
    const markdown = `Intro\n\n${fence(FLOW)}\n\ntext\n\n${fence(`\n${SEQUENCE}\n\n`)}\n`

    expect(mermaidSourcesOf(markdown)).toEqual([FLOW, SEQUENCE])
  })

  it('ignores a mermaid fence that is shown inside a code block', () => {
    const markdown = `\`\`\`\`markdown\n${fence(FLOW)}\n\`\`\`\`\n\n${fence(SEQUENCE)}`

    expect(mermaidSourcesOf(markdown)).toEqual([SEQUENCE])
  })

  it('keeps one of the sources that repeat', () => {
    expect(mermaidSourcesOf(`${fence(FLOW)}\n\n${fence(FLOW)}`)).toEqual([FLOW])
  })

  it('returns nothing for a page with no mermaid fence', () => {
    expect(mermaidSourcesOf('# Title\n\n```java\nclass A {}\n```')).toEqual([])
  })
})

describe('drawDiagramImages', () => {
  const png = (bytes: number): Uint8Array => new Uint8Array(bytes)

  it('draws each source and counts the progress', async () => {
    const render: DiagramRenderer = vi.fn(async () => png(10))
    const progress: DiagramProgress[] = []

    const images = await drawDiagramImages([FLOW, SEQUENCE], new AbortController().signal, (step) => progress.push(step), render)

    expect(images.map((image) => image.source)).toEqual([FLOW, SEQUENCE])
    expect(progress.map(({ finished, total }) => [finished, total])).toEqual([[1, 2], [2, 2]])
  })

  it('leaves out a source that fails and reports it without throwing', async () => {
    const render: DiagramRenderer = async (source) => {
      if (source === FLOW) throw new Error('Parse error on line 2')
      return png(10)
    }
    const progress: DiagramProgress[] = []

    const images = await drawDiagramImages([FLOW, SEQUENCE], new AbortController().signal, (step) => progress.push(step), render)

    expect(images.map((image) => image.source)).toEqual([SEQUENCE])
    expect(progress.at(-1)).toEqual({ finished: 2, total: 2, failures: [{ source: FLOW, reason: 'Parse error on line 2' }] })
  })

  it('redraws at 1x a picture over 2 MB', async () => {
    const render = vi.fn<DiagramRenderer>(async (_source, scale) => png(scale === 2 ? 3 * 1024 * 1024 : 1000))

    const images = await drawDiagramImages([FLOW], new AbortController().signal, () => undefined, render)

    expect(render.mock.calls.map(([, scale]) => scale)).toEqual([2, 1])
    expect(images[0]?.png.length).toBe(1000)
  })

  it('stops when aborted and draws nothing more', async () => {
    const controller = new AbortController()
    const render = vi.fn<DiagramRenderer>(async () => {
      controller.abort()
      return png(10)
    })

    await expect(drawDiagramImages([FLOW, SEQUENCE], controller.signal, () => undefined, render)).rejects.toThrow()
    expect(render).toHaveBeenCalledTimes(1)
  })
})
