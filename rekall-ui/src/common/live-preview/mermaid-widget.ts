import { WidgetType, type EditorView } from '@codemirror/view'
import mermaid from 'mermaid'

// The read view's settings, so a diagram looks the same while writing.
mermaid.initialize({ startOnLoad: false, theme: 'dark' })

// Drawing is slow and a decoration rebuild recreates the widget, so each source is drawn once.
const drawn = new Map<string, Promise<string>>()
let sequence = 0

/** A mermaid fence drawn as its diagram. */
export class MermaidWidget extends WidgetType {
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
    drawingOf(this.source).then(
      (svg) => {
        // mermaid sanitises its own output at the default security level.
        canvas.innerHTML = svg
        view.requestMeasure()
      },
      (failure: unknown) => {
        frame.classList.add('cm-rk-mermaid--failed')
        canvas.textContent = `Mermaid could not draw this diagram: ${failure instanceof Error ? failure.message : String(failure)}`
        view.requestMeasure()
      }
    )
    return frame
  }
  // A click places the caret here, which shows the source.
  override ignoreEvent(): boolean {
    return false
  }
}

function drawingOf(source: string): Promise<string> {
  const cached = drawn.get(source)
  if (cached) return cached
  const drawing = mermaid.render(`rk-live-mermaid-${++sequence}`, source).then((result) => result.svg)
  drawing.catch(() => drawn.delete(source))
  drawn.set(source, drawing)
  return drawing
}
