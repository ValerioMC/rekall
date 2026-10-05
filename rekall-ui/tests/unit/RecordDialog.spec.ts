import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import RecordDialog from '@/components/console/RecordDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import { projectDraft, taskDraft, type RecordDraft } from '@/model/record-draft'
import type { Project, Task } from '@/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

/**
 * The save button while a save is in flight: disabled and inert, with no spinner, because the
 * dialog usually closes before any animation could finish. The write path is stubbed.
 */
const projectId = 'p1' as ProjectId

let pinia: Pinia
let wrapper: VueWrapper | null = null

function pendingCreate(): { resolve: () => void; create: ReturnType<typeof vi.fn> } {
  let resolve: () => void = () => undefined
  const pending = new Promise<Task>((settle) => {
    resolve = () => settle({} as Task)
  })
  return { resolve, create: vi.fn().mockReturnValue(pending) }
}

function render(): VueWrapper {
  wrapper = mount(RecordDialog, {
    attachTo: document.body,
    props: { draft: { ...taskDraft(projectId), title: 'Report builder', label: 'report-builder' } },
    global: { plugins: [pinia] }
  })
  return wrapper
}

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('RecordDialog save button', () => {
  it('disables itself without a spinner while the save runs', async () => {
    const store = useConsoleStore()
    const { resolve, create } = pendingCreate()
    store.createTask = create
    const dialog = render()
    const save = dialog.find('[data-testid="record-save"]')

    await save.trigger('click')

    expect(save.attributes('disabled')).toBeDefined()
    expect(save.find('.animate-spin').exists()).toBe(false)
    resolve()
    await flushPromises()
    expect(dialog.emitted('close')).toHaveLength(1)
  })

  it('ignores a second submit while the first is still saving', async () => {
    const store = useConsoleStore()
    const { resolve, create } = pendingCreate()
    store.createTask = create
    const dialog = render()

    await dialog.find('[data-testid="record-save"]').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', metaKey: true }))
    resolve()
    await flushPromises()

    expect(create).toHaveBeenCalledTimes(1)
  })
})

describe('RecordDialog project icon image', () => {
  function renderProject(): VueWrapper {
    wrapper = mount(RecordDialog, {
      attachTo: document.body,
      props: { draft: { ...projectDraft('c1' as CompanyId), title: 'Vega', label: 'vega' } },
      global: { plugins: [pinia] }
    })
    return wrapper
  }

  async function pick(dialog: VueWrapper, file: File): Promise<void> {
    const input = dialog.get('[data-testid="record-icon-file"]')
    Object.defineProperty(input.element, 'files', { value: [file], configurable: true })
    await input.trigger('change')
  }

  it('uploads the picked file for the project it has just created', async () => {
    URL.createObjectURL = vi.fn().mockReturnValue('blob:preview')
    URL.revokeObjectURL = vi.fn()
    const store = useConsoleStore()
    store.createProject = vi.fn().mockResolvedValue({ id: 'p9' } as Project)
    store.setProjectIconImage = vi.fn().mockResolvedValue(undefined)
    const dialog = renderProject()
    const file = new File(['x'], 'logo.png', { type: 'image/png' })

    await pick(dialog, file)
    expect(dialog.get('[data-testid="project-mark-image"]').attributes('src')).toBe('blob:preview')
    await dialog.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(store.setProjectIconImage).toHaveBeenCalledWith('p9', file)
  })

  it('refuses a file over 10 MB without keeping it', async () => {
    const store = useConsoleStore()
    store.createProject = vi.fn().mockResolvedValue({ id: 'p9' } as Project)
    store.setProjectIconImage = vi.fn()
    const dialog = renderProject()
    const huge = new File(['x'], 'huge.png', { type: 'image/png' })
    Object.defineProperty(huge, 'size', { value: 11 * 1024 * 1024 })

    await pick(dialog, huge)
    await dialog.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(dialog.get('[role="alert"]').text()).toContain('10 MB')
    expect(store.setProjectIconImage).not.toHaveBeenCalled()
  })
})

describe('RecordDialog identity fields', () => {
  function renderNewTask(): VueWrapper {
    const store = useConsoleStore()
    store.projects = [{ id: projectId, anchor: 'project:vega' } as Project]
    wrapper = mount(RecordDialog, {
      attachTo: document.body,
      props: { draft: taskDraft(projectId) },
      global: { plugins: [pinia] }
    })
    return wrapper
  }

  it('fills the label from the title until the label is typed in', async () => {
    const dialog = renderNewTask()

    await dialog.get('[data-testid="record-title"]').setValue('Report Builder')
    expect((dialog.get('[data-testid="record-label"]').element as HTMLInputElement).value).toBe('report-builder')

    await dialog.get('[data-testid="record-label"]').setValue('rb')
    await dialog.get('[data-testid="record-title"]').setValue('Report Builder v2')
    expect((dialog.get('[data-testid="record-label"]').element as HTMLInputElement).value).toBe('rb')
  })

  it('previews the anchor the record will load under, parent first', async () => {
    const dialog = renderNewTask()

    await dialog.get('[data-testid="record-title"]').setValue('Report Builder')

    expect(dialog.get('[data-testid="anchor-preview"]').text()).toContain('project:vega task:report-builder')
  })

  it('warns that renaming the label moves the anchor of an existing record', async () => {
    const store = useConsoleStore()
    store.projects = [{ id: projectId, anchor: 'project:vega' } as Project]
    wrapper = mount(RecordDialog, {
      attachTo: document.body,
      props: { draft: { ...taskDraft(projectId), id: 't1', title: 'Report', label: 'report' } as RecordDraft },
      global: { plugins: [pinia] }
    })
    expect(wrapper.find('[data-testid="rename-warning"]').exists()).toBe(false)

    await wrapper.get('[data-testid="record-label"]').setValue('report-builder')

    expect(wrapper.get('[data-testid="rename-warning"]').text()).toContain('task:report')
  })

  it('refuses to save without a title and says why', async () => {
    const store = useConsoleStore()
    store.createTask = vi.fn()
    const dialog = renderNewTask()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', metaKey: true }))
    await flushPromises()

    expect(store.createTask).not.toHaveBeenCalled()
    expect(dialog.get('[role="alert"]').text()).toBe('A title is required.')
  })
})

describe('RecordDialog task tag and keys', () => {
  it('creates the task with the tag picked', async () => {
    const store = useConsoleStore()
    store.tags = [{ id: 'g1', name: 'Bug', icon: 'bug', color: 'red' } as (typeof store.tags)[number]]
    store.createTask = vi.fn().mockResolvedValue({} as Task)
    const dialog = render()

    const bug = dialog.findAll('[data-testid="record-tag-picker"] button').find((button) => button.text() === 'Bug')!
    await bug.trigger('click')
    await dialog.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(store.createTask).toHaveBeenCalledWith(expect.objectContaining({ tagId: 'g1', label: 'report-builder' }))
  })

  it('closes on Escape', async () => {
    const dialog = render()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(dialog.emitted('close')).toHaveLength(1)
  })

  it('focuses the title when it opens', async () => {
    render()
    await flushPromises()

    expect((document.activeElement as HTMLElement | null)?.dataset.testid).toBe('record-title')
  })
})
