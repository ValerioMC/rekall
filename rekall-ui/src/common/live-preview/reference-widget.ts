import { WidgetType, type EditorView } from '@codemirror/view'
import { referenceIconElement } from '@/common/markdown/reference-completion-icon'
import type { ReferenceChip } from '@/common/markdown/reference-chip'
import { caretInto } from '@/common/live-preview/editor-focus'

const CLOSING_LENGTH = 2

/** A `[[...]]` drawn as its chip: the glyph of what it names, then its name. A click shows the source with the caret in the name. */
export class ReferenceWidget extends WidgetType {
  constructor(
    readonly source: string,
    readonly chip: ReferenceChip
  ) {
    super()
  }

  override eq(other: ReferenceWidget): boolean {
    const { text, title, icon, missing } = this.chip
    return (
      other.source === this.source &&
      other.chip.text === text &&
      other.chip.title === title &&
      other.chip.icon === icon &&
      other.chip.missing === missing
    )
  }

  override toDOM(view: EditorView): HTMLElement {
    const element = document.createElement('span')
    element.className = ['cm-rk-ref', this.chip.icon === 'task' && 'cm-rk-ref--task', this.chip.missing && 'cm-rk-ref--missing']
      .filter(Boolean)
      .join(' ')
    element.title = this.chip.title
    if (this.chip.icon) element.append(referenceIconElement(this.chip.icon))
    element.append(this.chip.text)
    element.addEventListener('mousedown', (event) => {
      event.preventDefault()
      const start = view.posAtDOM(element)
      view.dispatch(caretInto(start + this.source.length - CLOSING_LENGTH))
    })
    return element
  }

  override ignoreEvent(): boolean {
    return true
  }
}
