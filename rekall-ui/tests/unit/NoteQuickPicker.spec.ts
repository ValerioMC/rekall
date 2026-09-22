import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NoteQuickPicker from '@/components/console/NoteQuickPicker.vue'
import NotesButton from '@/components/console/NotesButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { RekallDocument, TaskRef } from '@/model/catalog'
import type { DocumentId, TaskId } from '@/model/branded'

/**
 * The picker hung under the Notes button in the description and steps headers. It only decides
 * which task ids a note ends up on; the write path is `store.saveNote`, stubbed here and covered
 * in `console.spec`.
 */
const here = 't1' as TaskId
const elsewhere = 't2' as TaskId
const third = 't3' as TaskId

const ref = (id: TaskId, label: string): TaskRef => ({
  id,
  label,
  title: label,
  projectLabel: 'rekall',
  projectTitle: 'Rekall',
  companyName: 'vforge',
  anchor: `project:rekall task:${label}`
})

const refs: Record<TaskId, TaskRef> = {
  [here]: ref(here, 'note-improvement'),
  [elsewhere]: ref(elsewhere, 'filing-drawer'),
  [third]: ref(third, 'sse-conduit')
}

const doc = (id: string, title: string, on: TaskId[], updatedAt = '2026-09-01T10:00:00Z'): RekallDocument => ({
  id: id as DocumentId,
  title,
  kind: 'notes',
  bodyMarkdown: `${title} body`,
  tasks: on.map((taskId) => refs[taskId]!),
  contextMode: 'FULL',
  anchor: 'note:00000000',
  updatedAt
})

let pinia: Pinia

function seed(documents: RekallDocument[]) {
  const store = useConsoleStore()
  store.documents = documents
  store.selectedTaskId = here
  store.isLoading = false
  store.saveNote = vi.fn().mockResolvedValue(undefined)
  return store
}

function render() {
  const anchor = document.createElement('div')
  document.body.appendChild(anchor)
  return mount(NoteQuickPicker, {
    props: { taskId: here, anchor },
    attachTo: document.body,
    global: { plugins: [pinia] }
  })
}

const rowsOf = (wrapper: ReturnType<typeof render>) =>
  Array.from(document.body.querySelectorAll<HTMLElement>('[data-testid="note-picker-row"]')).filter(
    () => wrapper.exists()
  )

describe('NoteQuickPicker', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('lists the notes on this task first, then the rest', async () => {
    seed([
      doc('d1', 'conventions.md', [elsewhere], '2026-09-05T10:00:00Z'),
      doc('d2', 'cluster.md', [here, elsewhere], '2026-09-03T10:00:00Z'),
      doc('d3', 'gotchas.md', [here], '2026-09-01T10:00:00Z')
    ])
    const wrapper = render()
    await flushPromises()

    const rows = rowsOf(wrapper)
    expect(rows.map((row) => row.dataset.attached)).toEqual(['true', 'true', 'false'])
    expect(rows.map((row) => row.textContent)).toEqual([
      expect.stringContaining('cluster.md'),
      expect.stringContaining('gotchas.md'),
      expect.stringContaining('conventions.md')
    ])
    expect(document.body.querySelector('[data-testid="note-picker-count"]')?.textContent).toContain('2/3')
    wrapper.unmount()
  })

  it('adds a note to this task on click, keeping the tasks it was already on', async () => {
    const store = seed([doc('d1', 'conventions.md', [elsewhere, third])])
    const wrapper = render()
    await flushPromises()

    rowsOf(wrapper)[0]!.click()
    await flushPromises()

    expect(store.saveNote).toHaveBeenCalledWith('d1', { taskIds: [elsewhere, third, here] })
    wrapper.unmount()
  })

  it('removes a note that is also on another task', async () => {
    const store = seed([doc('d1', 'conventions.md', [here, elsewhere])])
    const wrapper = render()
    await flushPromises()

    rowsOf(wrapper)[0]!.click()
    await flushPromises()

    expect(store.saveNote).toHaveBeenCalledWith('d1', { taskIds: [elsewhere] })
    wrapper.unmount()
  })

  it('refuses to remove a note whose only task is this one', async () => {
    const store = seed([doc('d1', 'conventions.md', [here])])
    const wrapper = render()
    await flushPromises()

    const row = rowsOf(wrapper)[0]!
    expect(row.getAttribute('aria-disabled')).toBe('true')
    expect(row.querySelector('[data-testid="note-picker-elsewhere"]')?.textContent).toContain('only here')
    row.click()
    await flushPromises()

    expect(store.saveNote).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('says where else a note sits', async () => {
    seed([
      doc('d1', 'one.md', [here, elsewhere]),
      doc('d2', 'many.md', [here, elsewhere, third]),
      doc('d3', 'away.md', [elsewhere])
    ])
    const wrapper = render()
    await flushPromises()

    const notes = rowsOf(wrapper).map(
      (row) => row.querySelector('[data-testid="note-picker-elsewhere"]')?.textContent?.trim()
    )
    expect(notes).toEqual(['also on rekall/filing-drawer', 'also on 2 tasks', 'on rekall/filing-drawer'])
    wrapper.unmount()
  })

  it('filters by title and body, and walks the list with the arrow keys and Enter', async () => {
    const store = seed([
      doc('d1', 'conventions.md', [here]),
      doc('d2', 'cluster.md', [elsewhere]),
      doc('d3', 'gotchas.md', [elsewhere])
    ])
    const wrapper = render()
    await flushPromises()

    const filter = document.body.querySelector<HTMLInputElement>('[data-testid="note-picker-filter"]')!
    filter.value = 'gotchas'
    filter.dispatchEvent(new Event('input'))
    await flushPromises()
    expect(rowsOf(wrapper)).toHaveLength(1)

    filter.value = ''
    filter.dispatchEvent(new Event('input'))
    await flushPromises()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown' }))
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown' }))
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
    await flushPromises()

    expect(store.saveNote).toHaveBeenCalledWith('d3', { taskIds: [elsewhere, here] })
    wrapper.unmount()
  })

  it('closes on Escape and on a pointer down outside the panel', async () => {
    seed([doc('d1', 'conventions.md', [here])])
    const wrapper = render()
    await flushPromises()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(wrapper.emitted('close')).toHaveLength(1)

    document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    expect(wrapper.emitted('close')).toHaveLength(2)
    wrapper.unmount()
  })

  it('tells the empty store to press N', async () => {
    seed([])
    const wrapper = render()
    await flushPromises()

    expect(document.body.querySelector('[data-testid="note-picker-empty"]')?.textContent).toContain('N')
    wrapper.unmount()
  })
})

describe('NotesButton', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('carries how many notes are on the task and opens the picker under itself', async () => {
    seed([doc('d1', 'a.md', [here]), doc('d2', 'b.md', [here, elsewhere]), doc('d3', 'c.md', [elsewhere])])
    const wrapper = mount(NotesButton, {
      props: { taskId: here },
      attachTo: document.body,
      global: { plugins: [pinia] }
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="notes-count"]').text()).toBe('2')
    expect(document.body.querySelector('[data-testid="note-picker"]')).toBeNull()

    await wrapper.get('[data-testid="notes-open"]').trigger('click')
    expect(document.body.querySelector('[data-testid="note-picker"]')).not.toBeNull()

    await wrapper.get('[data-testid="notes-open"]').trigger('click')
    expect(document.body.querySelector('[data-testid="note-picker"]')).toBeNull()
    wrapper.unmount()
  })
})
