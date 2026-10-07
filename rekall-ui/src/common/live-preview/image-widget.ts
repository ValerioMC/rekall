import { WidgetType } from '@codemirror/view'

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

  override toDOM(): HTMLElement {
    const image = document.createElement('img')
    image.className = 'cm-rk-image'
    image.src = this.source
    image.alt = this.alt
    return image
  }
  // A click places the caret here, which shows the source.
  override ignoreEvent(): boolean {
    return false
  }
}
