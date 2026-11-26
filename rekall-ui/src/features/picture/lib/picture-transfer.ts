import { Prec, StateEffect, StateField, type Extension } from '@codemirror/state'
import { Decoration, EditorView, WidgetType } from '@codemirror/view'
import { allPlacesReleased, heldPlaces, type TextRange } from '@/features/picture/lib/held-places'
import { draggedPictureCount, droppedPictures, pastedPictures } from '@/features/picture/lib/transferred-pictures'
import type { TransferredPicture } from '@/features/picture/model/picture'

/** Takes the pictures pasted or dropped on an editor, to be written over `range`. */
export type PictureReceiver = (view: EditorView, pictures: readonly TransferredPicture[], range: TextRange) => void

/** Sent by the editor's owner when the record pictures go to changes; `null` turns pictures away. */
export const pictureReceiverChanged = StateEffect.define<PictureReceiver | null>()

const pictureReceiver = StateField.define<PictureReceiver | null>({
  create: () => null,
  update: (receiver, transaction) =>
    transaction.effects.reduce((current, effect) => (effect.is(pictureReceiverChanged) ? effect.value : current), receiver)
})

interface DragTarget {
  readonly at: number
  readonly count: number
}

const dragTargetChanged = StateEffect.define<DragTarget | null>()

/** The mark drawn where the dragged pictures would be written. */
class DropMarkWidget extends WidgetType {
  override eq(): boolean {
    return true
  }

  override toDOM(): HTMLElement {
    const mark = document.createElement('span')
    mark.className = 'cm-rk-drop-mark'
    mark.setAttribute('aria-hidden', 'true')
    return mark
  }
}

/** Written on the editor, where the style sheet reads its outline and hint from. */
function dropHint(count: number): string {
  return count > 1 ? `Drop to add ${count} pictures here` : 'Drop to add the picture here'
}

const dragTarget = StateField.define<DragTarget | null>({
  create: () => null,
  update(target, transaction) {
    const kept = target && { ...target, at: transaction.changes.mapPos(target.at) }
    return transaction.effects.reduce((current, effect) => (effect.is(dragTargetChanged) ? effect.value : current), kept)
  },
  provide: (field) => [
    EditorView.decorations.from(field, (target) =>
      target ? Decoration.set(Decoration.widget({ widget: new DropMarkWidget() }).range(target.at)) : Decoration.none
    ),
    EditorView.editorAttributes.from(field, (target): Record<string, string> =>
      target ? { 'data-rk-drop': dropHint(target.count) } : {}
    )
  ]
})

function receiverOf(view: EditorView): PictureReceiver | null {
  return view.state.readOnly ? null : view.state.field(pictureReceiver)
}

function show(view: EditorView, target: DragTarget | null): void {
  const current = view.state.field(dragTarget)
  if (current?.at === target?.at && current?.count === target?.count) return
  view.dispatch({ effects: dragTargetChanged.of(target) })
}

function positionUnder(view: EditorView, event: DragEvent): number {
  return view.posAtCoords({ x: event.clientX, y: event.clientY }, false)
}

function takePasted(event: ClipboardEvent, view: EditorView): boolean {
  const receive = receiverOf(view)
  const pictures = pastedPictures(event.clipboardData)
  if (!receive || pictures.length === 0) return false
  const { from, to } = view.state.selection.main
  receive(view, pictures, { from, to })
  return true
}

/** A drag that began on this editor moves its own text or pictures, which stays the editor's business. */
function transferHandlers(): Extension {
  let draggingOwnContent = false

  function hover(event: DragEvent, view: EditorView): boolean {
    const count = draggedPictureCount(event.dataTransfer)
    if (draggingOwnContent || !receiverOf(view) || count === 0 || !event.dataTransfer) return false
    event.dataTransfer.dropEffect = 'copy'
    show(view, { at: positionUnder(view, event), count })
    return true
  }

  function take(event: DragEvent, view: EditorView): boolean {
    show(view, null)
    const receive = receiverOf(view)
    const pictures = droppedPictures(event.dataTransfer)
    if (draggingOwnContent || !receive || pictures.length === 0) return false
    const at = positionUnder(view, event)
    view.dispatch({ selection: { anchor: at } })
    view.focus()
    receive(view, pictures, { from: at, to: at })
    return true
  }

  return EditorView.domEventHandlers({
    paste: takePasted,
    dragstart: () => {
      draggingOwnContent = true
      return false
    },
    dragend: () => {
      draggingOwnContent = false
      return false
    },
    dragenter: hover,
    dragover: hover,
    dragleave: (event, view) => {
      const entered = event.relatedTarget
      if (!(entered instanceof Node) || !view.contentDOM.contains(entered)) show(view, null)
      return false
    },
    drop: take
  })
}

/**
 * Pictures pasted or dropped on a writing surface. Inert until a receiver is sent; ahead of the
 * library's own paste handler, which swallows any clipboard holding a file.
 */
export function pictureTransfer(): Extension {
  return [pictureReceiver, heldPlaces, dragTarget, Prec.highest(transferHandlers())]
}

export function sendPictureReceiver(view: EditorView, receiver: PictureReceiver | null): void {
  view.dispatch({ effects: [allPlacesReleased.of(null), pictureReceiverChanged.of(receiver)] })
}
