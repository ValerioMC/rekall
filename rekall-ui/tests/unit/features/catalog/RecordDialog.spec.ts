import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import { companyDraft, projectDraft, taskDraft, type RecordDraft } from '@/features/catalog/model/record-draft'
import type { Company, Project, Task } from '@/features/catalog/model/catalog'
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
    props: { draft: { ...taskDraft(projectId), title: 'Report builder' } },
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

describe('RecordDialog project folder', () => {
  const existing = {
    id: 'p1',
    label: 'vega',
    title: 'Vega',
    description: null,
    status: 'ACTIVE',
    icon: 'folder',
    companyId: 'c1',
    blueprintMarkdown: null,
    repoFolder: '/stored/vega',
    autoCommit: false
  } as unknown as Project

  function renderProject(draft: RecordDraft): VueWrapper {
    wrapper = mount(RecordDialog, { attachTo: document.body, props: { draft }, global: { plugins: [pinia] } })
    return wrapper
  }

  it('creates the project with the folder that was typed', async () => {
    const store = useConsoleStore()
    store.createProject = vi.fn().mockResolvedValue({ id: 'p9' } as Project)
    const dialog = renderProject({ ...projectDraft('c1' as CompanyId), title: 'Vega', label: 'vega' } as RecordDraft)

    await dialog.get('[data-testid="record-project-folder"]').setValue('  /work/vega  ')
    await dialog.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(store.createProject).toHaveBeenCalledWith(expect.objectContaining({ repoFolder: '/work/vega' }))
  })

  it('creates the project without a folder when the field is left blank', async () => {
    const store = useConsoleStore()
    store.createProject = vi.fn().mockResolvedValue({ id: 'p9' } as Project)
    const dialog = renderProject({ ...projectDraft('c1' as CompanyId), title: 'Vega', label: 'vega' } as RecordDraft)

    await dialog.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(store.createProject).toHaveBeenCalledWith(expect.objectContaining({ repoFolder: null }))
  })

  it('does not ask for the folder when an existing project is edited, and keeps the stored one', async () => {
    const store = useConsoleStore()
    store.projects = [existing]
    store.updateProject = vi.fn().mockResolvedValue(undefined)
    const dialog = renderProject(projectDraft('c1' as CompanyId, existing))

    expect(dialog.find('[data-testid="record-project-folder"]').exists()).toBe(false)
    await dialog.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(store.updateProject).toHaveBeenCalledWith('p1', expect.objectContaining({ repoFolder: '/stored/vega' }))
  })
})

describe('RecordDialog company logo', () => {
  const acme: Company = {
    id: 'c1' as CompanyId,
    name: 'acme',
    description: null,
    logoImageVersion: 'v1',
    projectCount: 0,
    taskCount: 0,
    updatedAt: ''
  }

  function renderCompany(company?: Company): VueWrapper {
    wrapper = mount(RecordDialog, {
      attachTo: document.body,
      props: { draft: company ? companyDraft(company) : { ...companyDraft(), name: 'Globex' } },
      global: { plugins: [pinia] }
    })
    return wrapper
  }

  it('uploads the picked logo for the company it has just created', async () => {
    URL.createObjectURL = vi.fn().mockReturnValue('blob:logo')
    URL.revokeObjectURL = vi.fn()
    const store = useConsoleStore()
    store.createCompany = vi.fn().mockResolvedValue({ ...acme, id: 'c9' as CompanyId })
    store.setCompanyLogoImage = vi.fn().mockResolvedValue(undefined)
    const dialog = renderCompany()
    const file = new File(['x'], 'logo.png', { type: 'image/png' })
    const input = dialog.get('[data-testid="record-logo-file"]')
    Object.defineProperty(input.element, 'files', { value: [file], configurable: true })

    await input.trigger('change')
    expect(dialog.get('[data-testid="company-mark-image"] img').attributes('src')).toBe('blob:logo')
    await dialog.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(store.setCompanyLogoImage).toHaveBeenCalledWith('c9', file)
  })

  it('takes the stored logo off on save, and shows the initial meanwhile', async () => {
    const store = useConsoleStore()
    store.companies = [acme]
    store.updateCompany = vi.fn().mockResolvedValue(undefined)
    store.setCompanyLogoImage = vi.fn().mockResolvedValue(undefined)
    const dialog = renderCompany(acme)

    await dialog.get('[data-testid="record-logo-remove"]').trigger('click')
    expect(dialog.get('[data-testid="company-mark-initial"]').text()).toBe('A')
    await dialog.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(store.setCompanyLogoImage).toHaveBeenCalledWith('c1', null)
  })

  it('leaves the logo alone when it was not touched', async () => {
    const store = useConsoleStore()
    store.companies = [acme]
    store.updateCompany = vi.fn().mockResolvedValue(undefined)
    store.setCompanyLogoImage = vi.fn()
    const dialog = renderCompany(acme)

    await dialog.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(store.setCompanyLogoImage).not.toHaveBeenCalled()
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

  it('has no label to type for a task: its code is assigned', () => {
    const dialog = renderNewTask()

    expect(dialog.find('[data-testid="record-label"]').exists()).toBe(false)
  })

  it('previews the code the task will get: its project’s key and the next number', async () => {
    const store = useConsoleStore()
    store.projects = [{ id: projectId, anchor: 'project:vega', taskKey: 'VG' } as Project]
    store.tasks = [{ id: 't1', projectId, code: 'VG-4' } as Task, { id: 't2', projectId, code: 'VG-2' } as Task]
    wrapper = mount(RecordDialog, {
      attachTo: document.body,
      props: { draft: taskDraft(projectId) },
      global: { plugins: [pinia] }
    })

    expect(wrapper.get('[data-testid="anchor-preview"]').text()).toContain('task:VG-5')
  })

  it('shows the code a task already has, whatever the project holds after it', () => {
    const store = useConsoleStore()
    const stored = { id: 't1', projectId, code: 'VG-2' } as Task
    store.projects = [{ id: projectId, anchor: 'project:vega', taskKey: 'VG' } as Project]
    store.tasks = [stored, { id: 't9', projectId, code: 'VG-9' } as Task]
    wrapper = mount(RecordDialog, {
      attachTo: document.body,
      props: { draft: { ...taskDraft(projectId, stored), id: 't1' } as RecordDraft },
      global: { plugins: [pinia] }
    })

    expect(wrapper.get('[data-testid="anchor-preview"]').text()).toContain('task:VG-2')
  })

  it('fills a project’s label from its title until the label is typed in', async () => {
    wrapper = mount(RecordDialog, { attachTo: document.body, props: { draft: projectDraft('c1' as CompanyId) }, global: { plugins: [pinia] } })

    await wrapper.get('[data-testid="record-title"]').setValue('Report Builder')
    expect((wrapper.get('[data-testid="record-label"]').element as HTMLInputElement).value).toBe('report-builder')

    await wrapper.get('[data-testid="record-label"]').setValue('rb')
    await wrapper.get('[data-testid="record-title"]').setValue('Report Builder v2')
    expect((wrapper.get('[data-testid="record-label"]').element as HTMLInputElement).value).toBe('rb')
  })

  it('takes a project’s task key as two to four capital letters, whatever is typed', async () => {
    wrapper = mount(RecordDialog, { attachTo: document.body, props: { draft: projectDraft('c1' as CompanyId) }, global: { plugins: [pinia] } })

    await wrapper.get('[data-testid="record-key"]').setValue('r2k-12x')

    expect((wrapper.get('[data-testid="record-key"]').element as HTMLInputElement).value).toBe('RKX')
  })

  it('sends the task key typed, and nothing when it is left empty', async () => {
    const store = useConsoleStore()
    store.createProject = vi.fn().mockResolvedValue({ id: 'p9' } as Project)
    store.setProjectIconImage = vi.fn().mockResolvedValue(undefined)
    wrapper = mount(RecordDialog, {
      attachTo: document.body,
      props: { draft: { ...projectDraft('c1' as CompanyId), title: 'Vega', label: 'vega', taskKey: 'VG' } },
      global: { plugins: [pinia] }
    })

    await wrapper.get('[data-testid="record-save"]').trigger('click')
    await flushPromises()

    expect(store.createProject).toHaveBeenCalledWith(expect.objectContaining({ label: 'vega', taskKey: 'VG' }))
  })

  it('warns that renaming the label moves the anchor of an existing project', async () => {
    wrapper = mount(RecordDialog, {
      attachTo: document.body,
      props: { draft: { ...projectDraft('c1' as CompanyId), id: 'p1', title: 'Vega', label: 'vega' } as RecordDraft },
      global: { plugins: [pinia] }
    })
    expect(wrapper.find('[data-testid="rename-warning"]').exists()).toBe(false)

    await wrapper.get('[data-testid="record-label"]').setValue('vega-platform')

    expect(wrapper.get('[data-testid="rename-warning"]').text()).toContain('project:vega')
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

    expect(store.createTask).toHaveBeenCalledWith(expect.objectContaining({ tagId: 'g1' }))
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
