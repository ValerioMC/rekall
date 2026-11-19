import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useTaskEditor } from '@/features/catalog/composables/useTaskEditor'
import { useConsoleStore } from '@/stores/console.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

describe('useTaskEditor', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    useConsoleStore().projects = [{ id: 'p1' as ProjectId, title: 'Vega' } as Project, { id: 'p2' as ProjectId, title: 'Orion' } as Project]
    useTaskEditor().close()
  })

  it('opens a new task on the project named, with the title it was given', () => {
    const editor = useTaskEditor()

    editor.beginCreate({ projectId: 'p2' as ProjectId, title: 'Plan the export' })

    expect(editor.editing.value).toMatchObject({ kind: 'task', id: null, projectId: 'p2', title: 'Plan the export' })
  })

  it('opens a new task with an empty title on the first project in scope when none is named', () => {
    const editor = useTaskEditor()

    editor.beginCreate()

    expect(editor.editing.value).toMatchObject({ projectId: 'p1', title: '' })
  })

  it('is one draft for every caller, so the page that hosts the dialog shows what another page asked for', () => {
    useTaskEditor().beginCreate({ projectId: 'p1' as ProjectId, title: 'Shared' })

    expect(useTaskEditor().editing.value).toMatchObject({ title: 'Shared' })
  })
})
