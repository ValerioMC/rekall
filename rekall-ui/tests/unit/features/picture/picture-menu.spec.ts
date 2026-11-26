import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { PICTURE_MENU_EVENT } from '@/common/live-preview/image-widget'
import { livePreview } from '@/common/live-preview/live-preview'
import PictureContextMenu from '@/features/picture/components/PictureContextMenu.vue'
import { usePictureContextMenu } from '@/features/picture/composables/usePictureContextMenu'

const WEB = 'https://example.com/logo.png'
const KEPT = '/api/images/acme/web/web-12/login-flow-3f9a1c2b.png'
const DOC = `![One](${KEPT}) and ![Two](${WEB}#right "A title")`

let view: EditorView
let edits: number

function menu(): ReturnType<typeof usePictureContextMenu> {
  const built = usePictureContextMenu(ref({ getEditorView: () => view } as unknown as ExposeParam))
  view.dom.parentElement!.addEventListener(PICTURE_MENU_EVENT, built.open)
  return built
}

function rightClick(picture: number): MouseEvent {
  const image = view.contentDOM.querySelectorAll('img.cm-rk-image')[picture]!
  const event = new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: 30, clientY: 40 })
  image.dispatchEvent(event)
  return event
}

beforeEach(() => {
  view = new EditorView({
    parent: document.body.appendChild(document.createElement('div')),
    state: EditorState.create({
      doc: DOC,
      extensions: [
        markdown({ base: markdownLanguage }),
        livePreview(),
        EditorView.updateListener.of((update) => void (update.docChanged && edits++))
      ]
    })
  })
  edits = 0
})

afterEach(() => {
  view.destroy()
  document.body.innerHTML = ''
})

describe('the menu of a picture in the writing view', () => {
  it('keeps the picture drawn when the right button goes down on it', () => {
    const image = view.contentDOM.querySelector('img.cm-rk-image')!
    const press = new MouseEvent('mousedown', { bubbles: true, cancelable: true, button: 2 })

    image.dispatchEvent(press)

    expect(press.defaultPrevented).toBe(true)
  })

  it('rewrites only the clicked picture of a line holding two, as one undo step', () => {
    const picture = menu()

    expect(rightClick(1).defaultPrevented).toBe(true)
    expect(picture.request.value).toMatchObject({ x: 30, y: 40, address: `${WEB}#right` })
    picture.choose('center')

    expect(view.state.doc.toString()).toBe(`![One](${KEPT}) and ![Two](${WEB}#center "A title")`)
    expect(edits).toBe(1)
    expect(picture.request.value).toBeNull()

    rightClick(0)
    picture.choose('right')
    expect(view.state.doc.toString()).toBe(`![One](${KEPT}#right) and ![Two](${WEB}#center "A title")`)
  })

  it('changes nothing when the current alignment is chosen again', () => {
    const picture = menu()
    rightClick(0)

    picture.choose('left')

    expect(edits).toBe(0)
  })

  it('marks the current alignment and answers the one chosen', async () => {
    const request = { x: 10, y: 10, picturePosition: 0, address: `${KEPT}#center` }
    const wrapper = mount(PictureContextMenu, { props: { request }, attachTo: document.body })
    const items = wrapper.findAll('[role="menuitemradio"]')

    expect(items.map((item) => item.text())).toEqual(['Align left', 'Align centre', 'Align right'])
    expect(items.map((item) => item.attributes('aria-checked'))).toEqual(['false', 'true', 'false'])
    expect(document.activeElement).toBe(items[1]!.element)

    await items[2]!.trigger('click')
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(wrapper.emitted('choose')).toEqual([['right']])
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })
})
