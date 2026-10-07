import { WidgetType } from '@codemirror/view'

/** The read view's short dash in place of a bullet's `-`, `*` or `+`. */
export class BulletWidget extends WidgetType {
  override eq(): boolean {
    return true
  }

  override toDOM(): HTMLElement {
    const bullet = document.createElement('span')
    bullet.className = 'cm-rk-bullet'
    bullet.setAttribute('aria-hidden', 'true')
    return bullet
  }
}
