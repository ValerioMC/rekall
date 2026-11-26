import { createPinia, setActivePinia } from 'pinia'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope, ref, type EffectScope, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { ApiError } from '@/api/client'
import { usePictureInsert } from '@/features/picture/composables/usePictureInsert'
import type { PictureOwner } from '@/features/picture/model/picture'
import { useToastStore } from '@/stores/toast.store'

vi.mock('@/features/picture/api/pictures.api', () => ({ importPicture: vi.fn() }))
import { importPicture } from '@/features/picture/api/pictures.api'

const SOURCE = '/Users/me/Desktop/Login Flow.png'
const URL = '/api/images/acme/web/web-12/login-flow-3f9a1c2b.png'
const OWNER: PictureOwner = { kind: 'TASK', id: 't-1' }

let view: EditorView
let scope: EffectScope
let owner: Ref<PictureOwner | null>
let pathRoot: Ref<string | null>

function mountComposable(): ReturnType<typeof usePictureInsert> {
  const editor = ref({ getEditorView: () => view } as unknown as ExposeParam)
  scope = effectScope()
  return scope.run(() =>
    usePictureInsert(editor, { readonly: () => false, pathRoot: () => pathRoot.value, owner: () => owner.value })
  )!
}

function select(from: number, to: number): void {
  view.dispatch({ selection: { anchor: from, head: to } })
}

beforeEach(() => {
  setActivePinia(createPinia())
  view = new EditorView({ state: EditorState.create({ doc: 'See HERE below.' }), parent: document.body })
  owner = ref(OWNER)
  pathRoot = ref('/Users/me/code/app')
  vi.mocked(importPicture).mockReset()
  vi.mocked(importPicture).mockResolvedValue({ url: URL })
})

afterEach(() => {
  scope.stop()
  view.destroy()
})

describe('usePictureInsert', () => {
  it('writes the picture over the selection the picker was opened on', async () => {
    const picture = mountComposable()
    select(4, 8)
    picture.toggle()
    select(0, 0)

    await picture.insert(SOURCE)

    expect(importPicture).toHaveBeenCalledWith(SOURCE, OWNER)
    expect(view.state.doc.toString()).toBe(`See ![Login Flow](${URL}) below.`)
    expect(view.state.selection.main.head).toBe(`See ![Login Flow](${URL})`.length)
    expect(picture.open.value).toBe(false)
  })

  it('writes the chosen alignment at the end of the address, and starts left the next time', async () => {
    const picture = mountComposable()
    select(4, 8)
    picture.toggle()
    picture.alignment.value = 'right'

    await picture.insert(SOURCE)

    expect(view.state.doc.toString()).toBe(`See ![Login Flow](${URL}#right) below.`)
    picture.toggle()
    expect(picture.alignment.value).toBe('left')
  })

  it('escapes the brackets a file name carries into the description', async () => {
    const picture = mountComposable()
    picture.toggle()

    await picture.insert('/tmp/a [draft].svg')

    expect(view.state.doc.toString()).toContain(`![a \\[draft\\]](${URL})`)
  })

  it('shows the refusal of the server and writes nothing', async () => {
    vi.mocked(importPicture).mockRejectedValue(new ApiError(400, 'That file is not a picture.', ''))
    const picture = mountComposable()
    picture.toggle()

    await picture.insert(SOURCE)

    expect(view.state.doc.toString()).toBe('See HERE below.')
    expect(useToastStore().toasts.map((toast) => [toast.kind, toast.message])).toEqual([
      ['error', 'That file is not a picture.']
    ])
  })

  it('writes nothing into a text that replaced the one it was opened on', async () => {
    const picture = mountComposable()
    picture.toggle()
    const pending = picture.insert(SOURCE)
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: 'Another step.' } })

    await pending

    expect(view.state.doc.toString()).toBe('Another step.')
    expect(useToastStore().toasts[0]?.message).toContain('was not inserted')
  })

  it('reopens where a picture was last picked, until the project changes', async () => {
    const picture = mountComposable()
    picture.toggle()
    picture.visit('/Users/me/Desktop')
    expect(picture.start.value).toBeNull()

    await picture.insert(SOURCE)
    expect(picture.start.value).toBe('/Users/me/Desktop')

    pathRoot.value = '/Users/me/code/other'
    await Promise.resolve()
    expect(picture.start.value).toBeNull()
  })
})
