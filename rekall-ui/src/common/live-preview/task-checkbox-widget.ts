import { WidgetType, type EditorView } from '@codemirror/view'

/** A task's `[ ]` as a checkbox; ticking it writes `[x]` (or `[ ]`) into the text. */
export class TaskCheckboxWidget extends WidgetType {
  constructor(readonly checked: boolean) {
    super()
  }

  override eq(other: TaskCheckboxWidget): boolean {
    return other.checked === this.checked
  }

  override toDOM(view: EditorView): HTMLElement {
    const box = document.createElement('input')
    box.type = 'checkbox'
    box.className = 'cm-rk-task'
    box.checked = this.checked
    box.addEventListener('mousedown', (event) => {
      event.preventDefault()
      const marker = view.posAtDOM(box)
      view.dispatch({ changes: { from: marker + 1, to: marker + 2, insert: this.checked ? ' ' : 'x' } })
    })
    return box
  }

  override ignoreEvent(): boolean {
    return true
  }
}
