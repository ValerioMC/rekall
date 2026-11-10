import { WidgetType, type EditorView } from '@codemirror/view'

/** A mermaid fence drawn as the Diagram section draws it. */
export class MermaidWidget extends WidgetType {
  private readonly lifetime = new AbortController()

  constructor(readonly source: string) {
    super()
  }

  override eq(other: MermaidWidget): boolean {
    return other.source === this.source
  }

  override toDOM(view: EditorView): HTMLElement {
    const frame = document.createElement('div')
    frame.className = 'cm-rk-mermaid'
    const canvas = frame.appendChild(document.createElement('div'))
    canvas.className = 'cm-rk-mermaid-canvas'
    this.draw(canvas, view).catch((failure: unknown) => {
      frame.classList.add('cm-rk-mermaid--failed')
      canvas.textContent = `Mermaid could not draw this diagram: ${failure instanceof Error ? failure.message : String(failure)}`
      view.requestMeasure()
    })
    return frame
  }

  override destroy(): void {
    this.lifetime.abort()
  }

  // A click places the caret here, which shows the source.
  override ignoreEvent(): boolean {
    return false
  }

  private async draw(canvas: HTMLElement, view: EditorView): Promise<void> {
    const { showStaticDiagram } = await import('@/features/diagram/lib/static-diagram')
    await showStaticDiagram(canvas, this.source, this.lifetime.signal)
    view.requestMeasure()
  }
}
