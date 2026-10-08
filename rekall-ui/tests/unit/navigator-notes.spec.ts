import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, nextTick, ref, type Ref } from 'vue'
import NavigatorNoteGroup from '@/components/console/NavigatorNoteGroup.vue'
import NavigatorNoteRow from '@/components/console/NavigatorNoteRow.vue'
import { groupNotesByScope, type NoteGroup } from '@/common/catalog/note-groups'
import { useNoteShelves, type NoteShelves } from '@/composables/useNoteShelves'
import type { RekallDocument, TaskRef } from '@/model/catalog'
import type { DocumentId, ProjectId, TaskId } from '@/model/branded'

const vega = 'p1' as ProjectId

function taskRef(index: number): TaskRef {
  return {
    id: `t${index}` as TaskId,
    label: `task-${index}`,
    title: `Task ${index}`,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'Acme',
    anchor: `project:vega task:task-${index}`
  }
}

function note(id: string, onTasks: number, over: Partial<RekallDocument> = {}): RekallDocument {
  return {
    id: id as DocumentId,
    title: `${id}.md`,
    kind: 'notes',
    bodyMarkdown: '',
    tasks: Array.from({ length: onTasks }, (_, index) => taskRef(index)),
    contextMode: 'FULL',
    scope: { kind: 'PROJECT', id: vega },
    anchor: `note:${id}`,
    updatedAt: '2026-09-01T10:00:00Z',
    ...over
  }
}

function vegaGroup(notes: RekallDocument[]): NoteGroup {
  return groupNotesByScope(
    notes,
    [{ id: vega, title: 'Vega', icon: 'rocket', iconImageVersion: null, companyName: 'Acme' }],
    []
  )[0]!
}

/** A store of the test's own: this environment hands out no usable `localStorage`. */
function installStorage(): Map<string, string> {
  const values = new Map<string, string>()
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => void values.set(key, value),
      removeItem: (key: string) => void values.delete(key),
      clear: () => values.clear()
    }
  })
  return values
}

function shelvesFor(selected: Ref<RekallDocument | null>): NoteShelves {
  let shelves: NoteShelves | undefined
  mount(defineComponent({ setup: () => void (shelves = useNoteShelves(selected)), template: '<div />' }))
  return shelves!
}

function mountGroup(group: NoteGroup, shelves: NoteShelves, selectedNoteId: DocumentId | null = null) {
  return mount(NavigatorNoteGroup, {
    props: { group, shelves, selectedNoteId },
    global: { stubs: { RouterLink: { template: '<a><slot /></a>' } } }
  })
}

beforeEach(() => {
  installStorage()
})

describe('a note row', () => {
  it('names the first task it is on and counts the rest', () => {
    const wrapper = mount(NavigatorNoteRow, { props: { note: note('a', 3), selected: false, tint: null } })

    expect(wrapper.text()).toContain('task-0')
    expect(wrapper.text()).toContain('+2')
    expect(wrapper.find('[data-testid="note-row-reference"]').exists()).toBe(false)
  })

  it('marks a note sessions load only on request', () => {
    const wrapper = mount(NavigatorNoteRow, {
      props: { note: note('a', 1, { contextMode: 'REFERENCE' }), selected: false, tint: null }
    })

    expect(wrapper.find('[data-testid="note-row-reference"]').exists()).toBe(true)
  })
})

describe('a scope card in the notes navigator', () => {
  it('lists the notes on a task and keeps the loose ones on a closed shelf', async () => {
    const group = vegaGroup([note('placed', 1), note('loose', 0)])
    const wrapper = mountGroup(group, shelvesFor(ref(null)))

    expect(wrapper.find('[data-testid="note-group-name"]').text()).toBe('Vega')
    expect(wrapper.findAll('[data-testid="note-row"]').map((row) => row.text())).toEqual([
      expect.stringContaining('placed.md')
    ])
    expect(wrapper.find('[data-testid="filing-drawer-toggle"]').text()).toContain('1 on no task')

    await wrapper.find('[data-testid="filing-drawer-toggle"]').trigger('click')

    expect(wrapper.findAll('[data-testid="note-row"]')).toHaveLength(2)
  })

  it('folds the whole card away and says which note was picked', async () => {
    const group = vegaGroup([note('placed', 1)])
    const wrapper = mountGroup(group, shelvesFor(ref(null)))

    await wrapper.find('[data-testid="note-row"]').trigger('click')
    expect(wrapper.emitted('select')).toEqual([['placed']])

    await wrapper.find('[data-testid="note-group-toggle"]').trigger('click')
    expect(wrapper.find('[data-testid="note-group-toggle"]').attributes('aria-expanded')).toBe('false')
  })
})

describe('the notes navigator shelves', () => {
  it('opens the loose shelf of a loose note picked from elsewhere', async () => {
    const selected = ref<RekallDocument | null>(null)
    const shelves = shelvesFor(selected)

    selected.value = note('loose', 0)
    await nextTick()

    expect(shelves.openLoose.has(`PROJECT:${vega}`)).toBe(true)
  })

  it('remembers a folded card across a restart', () => {
    const first = shelvesFor(ref(null))
    first.toggleCollapsed('GLOBAL')

    expect(shelvesFor(ref(null)).collapsedGroups.has('GLOBAL')).toBe(true)
  })
})
