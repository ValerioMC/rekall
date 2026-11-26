import { StateEffect, StateField, Transaction, type ChangeDesc } from '@codemirror/state'
import { Decoration, EditorView, WidgetType } from '@codemirror/view'

export interface TextRange {
  readonly from: number
  readonly to: number
}

interface HeldPlace extends TextRange {
  readonly id: number
  readonly label: string
}

const placeHeld = StateEffect.define<HeldPlace>()
const placeRelabelled = StateEffect.define<{ id: number; label: string }>()
const placeReleased = StateEffect.define<number>()
export const allPlacesReleased = StateEffect.define<null>()

let lastPlaceId = 0

/** The pill that stands where a picture will be written, while its file is copied. */
class CopyingWidget extends WidgetType {
  constructor(readonly label: string) {
    super()
  }

  override eq(other: CopyingWidget): boolean {
    return other.label === this.label
  }

  override toDOM(): HTMLElement {
    const pill = document.createElement('span')
    pill.className = 'cm-rk-copying'
    pill.setAttribute('role', 'status')
    const spinner = pill.appendChild(document.createElement('span'))
    spinner.className = 'cm-rk-copying-spinner'
    spinner.setAttribute('aria-hidden', 'true')
    const text = pill.appendChild(document.createElement('span'))
    text.className = 'cm-rk-copying-label'
    text.textContent = this.label
    return pill
  }
}

/** The editor swapping in another text, in which a place held in the old one means nothing. */
function replacesTheText(transaction: Transaction): boolean {
  if (!transaction.docChanged || transaction.annotation(Transaction.userEvent) !== undefined) return false
  let replaced = false
  transaction.changes.iterChanges((from, to) => {
    replaced ||= from === 0 && to === transaction.startState.doc.length
  })
  return replaced
}

/** Text typed at a held place lands after the picture; a selection edited meanwhile is no longer replaced. */
function mapped(place: HeldPlace, changes: ChangeDesc): HeldPlace {
  const from = changes.mapPos(place.from, -1)
  const edited = place.from < place.to && changes.touchesRange(place.from, place.to) !== false
  return { ...place, from, to: edited ? from : changes.mapPos(place.to, -1) }
}

export const heldPlaces = StateField.define<readonly HeldPlace[]>({
  create: () => [],
  update(places, transaction) {
    const kept = replacesTheText(transaction) ? [] : places.map((place) => mapped(place, transaction.changes))
    return transaction.effects.reduce<readonly HeldPlace[]>((current, effect) => {
      if (effect.is(placeHeld)) return [...current, effect.value]
      if (effect.is(placeReleased)) return current.filter((place) => place.id !== effect.value)
      if (effect.is(allPlacesReleased)) return []
      if (!effect.is(placeRelabelled)) return current
      return current.map((place) => (place.id === effect.value.id ? { ...place, label: effect.value.label } : place))
    }, kept)
  },
  provide: (field) =>
    EditorView.decorations.from(field, (places) =>
      Decoration.set(
        places.map((place) => Decoration.widget({ widget: new CopyingWidget(place.label), side: -1 }).range(place.from)),
        true
      )
    )
})

export function holdPlace(view: EditorView, range: TextRange, label: string): number {
  const id = ++lastPlaceId
  view.dispatch({ effects: placeHeld.of({ id, label, from: range.from, to: range.to }) })
  return id
}

/** False once the place is gone: the text it was held in has been replaced. */
export function relabelPlace(view: EditorView, id: number, label: string): boolean {
  view.dispatch({ effects: placeRelabelled.of({ id, label }) })
  return view.state.field(heldPlaces).some((place) => place.id === id)
}

/**
 * Writes `text` into a held place as one undo step, moving a caret still waiting there past it;
 * empty text only releases the place. False once the place is gone.
 */
export function fillPlace(view: EditorView, id: number, text: string): boolean {
  const place = view.state.field(heldPlaces).find((candidate) => candidate.id === id)
  if (!place) return false
  const release = placeReleased.of(id)
  if (!text) {
    view.dispatch({ effects: release })
    return true
  }
  const { from, to } = view.state.selection.main
  const caretWaits = from === place.from && to === place.to
  view.dispatch({
    changes: { from: place.from, to: place.to, insert: text },
    ...(caretWaits ? { selection: { anchor: place.from + text.length }, scrollIntoView: true } : {}),
    effects: release,
    userEvent: 'input.paste'
  })
  return true
}
