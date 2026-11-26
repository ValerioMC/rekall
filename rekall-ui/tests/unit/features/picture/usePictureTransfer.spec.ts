import { createPinia, setActivePinia } from 'pinia'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, ref, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { ApiError } from '@/api/client'
import { usePictureTransfer } from '@/features/picture/composables/usePictureTransfer'
import { pictureTransfer } from '@/features/picture/lib/picture-transfer'
import type { ImportedPicture, PictureOwner } from '@/features/picture/model/picture'
import { useToastStore } from '@/stores/toast.store'

vi.mock('@/features/picture/api/pictures.api', () => ({ uploadPicture: vi.fn() }))
import { uploadPicture } from '@/features/picture/api/pictures.api'

const OWNER: PictureOwner = { kind: 'TASK', id: 't-1' }
const TEXT = 'See HERE below.'

let view: EditorView
let host: VueWrapper
let owner: Ref<PictureOwner | null>
/** The transactions that changed the text: one of them is one undo step. */
let edits: number

function url(name: string): string {
  return `/api/images/acme/web/web-12/${name}-3f9a1c2b.png`
}

function picture(name: string, type = 'image/png'): File {
  return new File(['pixels'], name, { type })
}

/** An upload the test finishes itself, to act while the file is still being copied. */
function pendingUpload(): { finish: (picture: ImportedPicture) => void } {
  let resolve: (picture: ImportedPicture) => void = () => undefined
  vi.mocked(uploadPicture).mockReturnValueOnce(new Promise((settle) => (resolve = settle)))
  return { finish: (picture) => resolve(picture) }
}

function transfer(files: File[], text = ''): DataTransfer {
  return {
    files,
    items: files.map((file) => ({ kind: 'file', type: file.type })),
    types: files.length > 0 ? ['Files'] : ['text/plain'],
    getData: () => text,
    dropEffect: 'none'
  } as unknown as DataTransfer
}

function fire(type: string, properties: Record<string, unknown>): Event {
  const event = new Event(type, { bubbles: true, cancelable: true })
  Object.entries(properties).forEach(([name, value]) => Object.defineProperty(event, name, { value }))
  view.contentDOM.dispatchEvent(event)
  return event
}

function paste(files: File[], text = ''): Event {
  return fire('paste', { clipboardData: transfer(files, text) })
}

function drag(type: 'dragover' | 'drop', files: File[]): Event {
  return fire(type, { dataTransfer: transfer(files), clientX: 10, clientY: 10 })
}

function text(): string {
  return view.state.doc.toString()
}

function pills(): string[] {
  return Array.from(view.dom.querySelectorAll('.cm-rk-copying'), (pill) => pill.textContent ?? '')
}

async function mountEditor(into: PictureOwner | null = OWNER): Promise<void> {
  view = new EditorView({
    state: EditorState.create({
      doc: TEXT,
      selection: { anchor: 4 },
      extensions: [pictureTransfer(), EditorView.updateListener.of((update) => void (update.docChanged && edits++))]
    }),
    parent: document.body
  })
  edits = 0
  owner = ref(into)
  host = mount(
    defineComponent({
      setup() {
        usePictureTransfer(ref({ getEditorView: () => view } as unknown as ExposeParam), () => owner.value)
        return () => null
      }
    })
  )
  await flushPromises()
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.mocked(uploadPicture).mockReset()
  vi.mocked(uploadPicture).mockImplementation(async (_file, name) => ({ url: url(name ? 'login-flow' : 'pasted-image') }))
})

afterEach(() => {
  host.unmount()
  view.destroy()
})

describe('pasting a picture', () => {
  it('copies a screenshot into the owner folder and writes it at the caret, which ends up after it', async () => {
    await mountEditor()
    const file = picture('image.png')

    const event = paste([file])
    await flushPromises()

    expect(event.defaultPrevented).toBe(true)
    expect(uploadPicture).toHaveBeenCalledWith(file, null, OWNER)
    expect(text()).toBe(`See ![pasted-image](${url('pasted-image')})HERE below.`)
    expect(view.state.selection.main.head).toBe(`See ![pasted-image](${url('pasted-image')})`.length)
    expect(pills()).toEqual([])
  })

  it('holds the place while copying, so text typed meanwhile lands after the picture', async () => {
    await mountEditor()
    const pending = pendingUpload()

    paste([picture('Login Flow.png')])
    expect(pills()).toEqual(['Copying Login Flow.png'])
    view.dispatch({ changes: { from: 4, insert: 'and ' }, selection: { anchor: 8 }, userEvent: 'input.type' })
    view.dispatch({ changes: { from: 0, insert: '> ' }, userEvent: 'input.type' })
    pending.finish({ url: url('login-flow') })
    await flushPromises()

    expect(text()).toBe(`> See ![Login Flow](${url('login-flow')})and HERE below.`)
    expect(view.state.selection.main.head).toBe(text().indexOf('HERE'))
  })

  it('writes several pictures in the order they came, as one undo step', async () => {
    await mountEditor()
    const first = pendingUpload()

    paste([picture('One.png'), picture('Two.png')])
    expect(pills()).toEqual(['Copying picture 1 of 2'])
    const second = pendingUpload()
    first.finish({ url: url('one') })
    await flushPromises()
    expect(pills()).toEqual(['Copying picture 2 of 2'])
    second.finish({ url: url('two') })
    await flushPromises()

    expect(text()).toBe(`See ![One](${url('one')})\n\n![Two](${url('two')})HERE below.`)
    expect(edits).toBe(1)
  })

  it('replaces the selection, and keeps what was typed over it meanwhile', async () => {
    await mountEditor()
    view.dispatch({ selection: { anchor: 4, head: 8 } })
    paste([picture('One.png')])
    await flushPromises()
    expect(text()).toBe(`See ![One](${url('login-flow')}) below.`)

    view.dispatch({ selection: { anchor: 0, head: 3 } })
    const pending = pendingUpload()
    paste([picture('Two.png')])
    view.dispatch({ changes: { from: 0, to: 3, insert: 'Open' }, userEvent: 'input.type' })
    pending.finish({ url: url('two') })
    await flushPromises()

    expect(text().startsWith(`![Two](${url('two')})Open ![One]`)).toBe(true)
  })

  it('leaves pasted text to the editor', async () => {
    await mountEditor()

    paste([], 'right ')
    await flushPromises()

    expect(text()).toBe('See right HERE below.')
    expect(uploadPicture).not.toHaveBeenCalled()
  })

  it('takes nothing in a text with no owner', async () => {
    await mountEditor(null)

    paste([picture('One.png')])
    drag('drop', [picture('One.png', 'text/plain')])
    await flushPromises()

    expect(uploadPicture).not.toHaveBeenCalled()
  })

  it('shows the refusal, writes nothing and keeps the selection', async () => {
    vi.mocked(uploadPicture).mockRejectedValue(new ApiError(400, 'The picture is larger than 10 MB.', ''))
    await mountEditor()
    view.dispatch({ selection: { anchor: 4, head: 8 } })

    paste([picture('Huge.png')])
    await flushPromises()

    expect(text()).toBe(TEXT)
    expect(pills()).toEqual([])
    expect(useToastStore().toasts.map((toast) => [toast.kind, toast.message])).toEqual([
      ['error', 'The picture is larger than 10 MB.']
    ])
  })

  it('writes nothing into a text that replaced the one it was pasted in', async () => {
    await mountEditor()
    const pending = pendingUpload()
    paste([picture('One.png')])

    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: 'Another step.' } })
    pending.finish({ url: url('one') })
    await flushPromises()

    expect(text()).toBe('Another step.')
    expect(pills()).toEqual([])
    expect(useToastStore().toasts[0]?.message).toContain('was not inserted')
  })

  it('writes nothing once the editor belongs to another record', async () => {
    await mountEditor()
    const pending = pendingUpload()
    paste([picture('One.png')])

    owner.value = { kind: 'TASK', id: 't-2' }
    await flushPromises()
    pending.finish({ url: url('one') })
    await flushPromises()

    expect(text()).toBe(TEXT)
    expect(useToastStore().toasts[0]?.message).toContain('was not inserted')
  })

  it('keeps its place when the owner is handed over again unchanged', async () => {
    await mountEditor()
    const pending = pendingUpload()
    paste([picture('One.png')])

    owner.value = { ...OWNER }
    await flushPromises()
    pending.finish({ url: url('one') })
    await flushPromises()

    expect(text()).toContain(`![One](${url('one')})`)
  })
})

describe('dropping a picture', () => {
  it('says where it will land while dragged and writes it there', async () => {
    await mountEditor()
    vi.spyOn(view, 'posAtCoords').mockReturnValue(9)

    const over = drag('dragover', [picture('One.png'), picture('Two.png')])
    expect(over.defaultPrevented).toBe(true)
    expect(view.dom.getAttribute('data-rk-drop')).toBe('Drop to add 2 pictures here')
    expect(view.dom.querySelector('.cm-rk-drop-mark')).not.toBeNull()

    drag('drop', [picture('One.png')])
    await flushPromises()

    expect(view.dom.hasAttribute('data-rk-drop')).toBe(false)
    expect(view.dom.querySelector('.cm-rk-drop-mark')).toBeNull()
    expect(uploadPicture).toHaveBeenCalledWith(expect.any(File), 'One.png', OWNER)
    expect(text()).toBe(`See HERE ![One](${url('login-flow')})below.`)
    expect(view.state.selection.main.head).toBe(text().indexOf('below'))
  })

  it('clears the mark when the drag leaves the editor', async () => {
    await mountEditor()
    vi.spyOn(view, 'posAtCoords').mockReturnValue(9)
    drag('dragover', [picture('One.png')])

    fire('dragleave', { relatedTarget: document.body })

    expect(view.dom.hasAttribute('data-rk-drop')).toBe(false)
  })

  it('offers no target to a file that is no picture, or to what was dragged out of the editor itself', async () => {
    await mountEditor()
    vi.spyOn(view, 'posAtCoords').mockReturnValue(9)

    drag('dragover', [new File(['a'], 'notes.txt', { type: 'text/plain' })])
    expect(view.dom.hasAttribute('data-rk-drop')).toBe(false)

    fire('dragstart', { dataTransfer: { ...transfer([]), setData: () => undefined, clearData: () => undefined } })
    drag('dragover', [picture('One.png')])
    expect(view.dom.hasAttribute('data-rk-drop')).toBe(false)
    drag('drop', [picture('One.png')])
    await flushPromises()
    expect(uploadPicture).not.toHaveBeenCalled()
  })
})
