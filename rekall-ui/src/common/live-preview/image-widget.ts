import { WidgetType, type EditorView } from '@codemirror/view'

/** Raised on the editor when a drawn picture is right-clicked; the editor's owner shows the menu. */
export const PICTURE_MENU_EVENT = 'rk-picture-menu'

export interface PictureMenuRequest {
  x: number
  y: number
  picturePosition: number
  address: string
}

/** The right button, or Control with the left one as macOS takes it. */
function asksForMenu(event: Event): boolean {
  if (event.type === 'contextmenu') return true
  return event instanceof MouseEvent && (event.button === 2 || (event.button === 0 && event.ctrlKey))
}

/** An image drawn in place of its `![alt](src)`. */
export class ImageWidget extends WidgetType {
  constructor(
    readonly source: string,
    readonly alt: string
  ) {
    super()
  }

  override eq(other: ImageWidget): boolean {
    return other.source === this.source && other.alt === this.alt
  }

  override toDOM(view: EditorView): HTMLElement {
    const image = document.createElement('img')
    image.className = 'cm-rk-image'
    image.src = this.source
    image.alt = this.alt
    this.listenForMenu(image, view)
    return image
  }

  // A click places the caret here, which shows the source; a right-click must leave the picture drawn.
  override ignoreEvent(event: Event): boolean {
    return asksForMenu(event)
  }

  private listenForMenu(image: HTMLElement, view: EditorView): void {
    image.addEventListener('mousedown', (event) => {
      if (asksForMenu(event)) event.preventDefault()
    })
    image.addEventListener('contextmenu', (event) => {
      event.preventDefault()
      const detail = { x: event.clientX, y: event.clientY, picturePosition: view.posAtDOM(image), address: this.source }
      image.dispatchEvent(new CustomEvent<PictureMenuRequest>(PICTURE_MENU_EVENT, { bubbles: true, detail }))
    })
  }
}
