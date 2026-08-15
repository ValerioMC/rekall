import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import App from '@/App.vue'
import type { Company, Project, RekallDocument, Task } from '@/model/catalog'
import type { CompanyId, DocumentId, ProjectId, TaskId } from '@/model/branded'

/**
 * The console as it actually renders, against a stubbed server. This catches the failures the
 * store tests cannot see: a pane that never mounts, a selection that reaches the wrong pane,
 * a keyboard shortcut wired to nothing, a form that sends the wrong field.
 */
const acme = 'c1' as CompanyId
const vega = 'p1' as ProjectId
const validator = 't1' as TaskId
const retry = 't2' as TaskId

const companies: Company[] = [
  { id: acme, name: 'acme', description: null, projectCount: 1, taskCount: 2, updatedAt: '2026-08-12T10:00:00Z' }
]

const beacon = 'p2' as ProjectId

const projects: Project[] = [
  { id: vega, label: 'vega', title: 'Vega Platform', status: 'ACTIVE', description: null, companyId: acme, companyName: 'acme', taskCount: 2, anchor: 'project:vega', updatedAt: '2026-08-12T10:00:00Z' },
  { id: beacon, label: 'beacon', title: 'Beacon', status: 'ACTIVE', description: null, companyId: acme, companyName: 'acme', taskCount: 0, anchor: 'project:beacon', updatedAt: '2026-08-12T10:00:00Z' }
]

const tasks: Task[] = [
  { id: validator, label: 'report-builder', title: 'Report builder', status: 'IN_PROGRESS', description: null, projectId: vega, projectLabel: 'vega', projectTitle: 'Vega Platform', companyName: 'acme', documentCount: 1, anchor: 'project:vega task:report-builder', updatedAt: '2026-08-12T10:00:00Z' },
  { id: retry, label: 'retry-policy', title: 'Retry policy', status: 'TODO', description: null, projectId: vega, projectLabel: 'vega', projectTitle: 'Vega Platform', companyName: 'acme', documentCount: 1, anchor: 'project:vega task:retry-policy', updatedAt: '2026-08-12T10:00:00Z' }
]

const shared: RekallDocument = {
  id: 'd1' as DocumentId,
  title: 'kmaster14.md',
  kind: 'notes',
  bodyMarkdown: '# Cluster\n\nAccesso via bastion.',
  tasks: [
    { id: validator, label: 'report-builder', title: 'Report builder', projectLabel: 'vega', projectTitle: 'Vega Platform', companyName: 'acme', anchor: 'project:vega task:report-builder' },
    { id: retry, label: 'retry-policy', title: 'Retry policy', projectLabel: 'vega', projectTitle: 'Vega Platform', companyName: 'acme', anchor: 'project:vega task:retry-policy' }
  ],
  updatedAt: '2026-08-12T13:00:00Z'
}

const updateTask = vi.fn(async () => ({ ...tasks[0]!, status: 'DONE' as const }))
const createTask = vi.fn(async () => ({ ...tasks[0]!, id: 't9' as TaskId }))
const deleteTask = vi.fn(async () => undefined)
const updateProject = vi.fn(async () => projects[0]!)

vi.mock('@/api/catalog.api', () => ({
  fetchCompanies: vi.fn(async () => companies),
  fetchProjects: vi.fn(async () => projects),
  createCompany: vi.fn(),
  updateCompany: vi.fn(),
  deleteCompany: vi.fn(),
  fetchTasks: vi.fn(async () => tasks),
  createProject: vi.fn(),
  updateProject: (...args: unknown[]) => updateProject(...(args as [])),
  deleteProject: vi.fn(),
  createTask: (...args: unknown[]) => createTask(...(args as [])),
  updateTask: (...args: unknown[]) => updateTask(...(args as [])),
  deleteTask: (...args: unknown[]) => deleteTask(...(args as []))
}))

vi.mock('@/api/documents.api', () => ({
  fetchAllDocuments: vi.fn(async () => [shared]),
  createDocument: vi.fn(),
  updateDocument: vi.fn(),
  deleteDocument: vi.fn()
}))

async function mountConsole() {
  setActivePinia(createPinia())
  const wrapper = mount(App, { attachTo: document.body })
  await flushPromises()
  return wrapper
}

describe('the console', () => {
  beforeEach(() => {
    updateTask.mockClear()
    createTask.mockClear()
    deleteTask.mockClear()
    updateProject.mockClear()
    document.body.innerHTML = ''
  })

  it('opens on the resume screen rather than an empty page', async () => {
    const wrapper = await mountConsole()

    expect(wrapper.text()).toContain('Pick up where you left off')
    expect(wrapper.findAll('[data-testid="resume-row"]')).toHaveLength(1)
  })

  it('renders the three panes with the project scope fixed above the list', async () => {
    const wrapper = await mountConsole()

    expect(wrapper.find('[data-testid="anchor-input"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('All work')
    expect(wrapper.findAll('[data-testid="task-row"]')).toHaveLength(2)
  })

  /** A row has to carry both names: the one you read and the one you type. */
  it('shows a task by its title with its label underneath', async () => {
    const wrapper = await mountConsole()
    const row = wrapper.findAll('[data-testid="task-row"]')[0]!

    expect(row.text()).toContain('Report builder')
    expect(row.text()).toContain('report-builder')
  })

  it('walks from a task to its notes to the editor', async () => {
    const wrapper = await mountConsole()

    await wrapper.findAll('[data-testid="task-row"]')[0]!.trigger('click')
    await flushPromises()

    expect(wrapper.findAll('[data-testid="note-card"]')).toHaveLength(1)
    expect(wrapper.text()).toContain('kmaster14.md')
    // The anchor is on screen and is the string you would paste after /rk.
    expect(wrapper.text()).toContain('project:vega task:report-builder')
  })

  /** The point of the model change has to be visible, not merely stored. */
  it('says when a note is shared, and names the other task', async () => {
    const wrapper = await mountConsole()

    await wrapper.findAll('[data-testid="task-row"]')[0]!.trigger('click')
    await flushPromises()

    expect(wrapper.text()).toContain('On 2 tasks')
    expect(wrapper.text()).toContain('vega/retry-policy')
    expect(wrapper.text()).toContain('Editing here changes what that other task load')
  })

  it('filters the task list as the anchor bar is typed into', async () => {
    const wrapper = await mountConsole()

    await wrapper.find('[data-testid="anchor-input"]').setValue('retry')
    await flushPromises()

    const rows = wrapper.findAll('[data-testid="task-row"]')
    expect(rows).toHaveLength(1)
    expect(rows[0]!.text()).toContain('Retry policy')
  })

  it('switches between browsing tasks and browsing notes on B', async () => {
    const wrapper = await mountConsole()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'b' }))
    await flushPromises()

    expect(wrapper.findAll('[data-testid="note-row"]')).toHaveLength(1)
    expect(wrapper.findAll('[data-testid="task-row"]')).toHaveLength(0)
  })

  it('sets a task status from the number keys, with no form and no save', async () => {
    const wrapper = await mountConsole()

    await wrapper.findAll('[data-testid="task-row"]')[0]!.trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: '4' }))
    await flushPromises()

    expect(updateTask).toHaveBeenCalledWith(validator, expect.objectContaining({ status: 'DONE' }))
  })

  /**
   * Four levels in one control. A select per level would put three controls above the list and
   * make choosing a project a two-step act; this shows the whole tree and takes one click.
   */
  it('shows companies with their projects nested in one picker', async () => {
    const wrapper = await mountConsole()

    await wrapper.find('[data-testid="scope-trigger"]').trigger('click')

    expect(wrapper.findAll('[data-testid="scope-company"]')).toHaveLength(1)
    expect(wrapper.findAll('[data-testid="scope-project"]')).toHaveLength(2)
    expect(wrapper.find('[data-testid="scope-company"]').text()).toContain('acme')
    expect(wrapper.find('[data-testid="scope-project"]').text()).toContain('Vega Platform')
    expect(wrapper.find('[data-testid="scope-project"]').text()).toContain('project:vega')
  })

  it('narrows to a project and says so in the breadcrumb', async () => {
    const wrapper = await mountConsole()

    await wrapper.find('[data-testid="scope-trigger"]').trigger('click')
    await wrapper.find('[data-testid="scope-project"]').trigger('click')
    await flushPromises()

    expect(wrapper.find('[data-testid="scope-trigger"]').text()).toContain('Vega Platform')
    expect(wrapper.find('[data-testid="scope-trigger"]').text()).toContain('project:vega')
    expect(wrapper.findAll('[data-testid="task-row"]')).toHaveLength(2)
  })

  it('offers creating a company and a project where projects are chosen', async () => {
    const wrapper = await mountConsole()

    await wrapper.find('[data-testid="scope-trigger"]').trigger('click')

    expect(wrapper.find('[data-testid="new-company"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="new-project"]').text()).toContain('acme')

    await wrapper.find('[data-testid="new-company"]').trigger('click')
    expect(wrapper.find('[data-testid="record-dialog"]').exists()).toBe(true)
    // A company has one name and no anchor label of its own.
    expect(wrapper.find('[data-testid="record-label"]').exists()).toBe(false)
  })

  /** Shortcuts must be inert while writing, or typing a note would navigate away from it. */
  it('ignores shortcuts while a field has focus', async () => {
    const wrapper = await mountConsole()
    const input = wrapper.find('[data-testid="anchor-input"]')
    ;(input.element as HTMLInputElement).focus()

    input.element.dispatchEvent(new KeyboardEvent('keydown', { key: 'b', bubbles: true }))
    await flushPromises()

    expect(wrapper.findAll('[data-testid="task-row"]').length).toBeGreaterThan(0)
  })

  describe('creating, editing and deleting a record', () => {
    /**
     * Typing a title and getting the label for free is most of what makes the two fields
     * bearable. The anchor is assembled while you type, so nobody saves a record to find out
     * what it became.
     */
    it('fills the label from the title and shows the anchor before anything is saved', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()

      await wrapper.find('[data-testid="record-title"]').setValue('Retry policy, main workflow')
      await flushPromises()

      expect((wrapper.find('[data-testid="record-label"]').element as HTMLInputElement).value)
        .toBe('retry-policy-main-workflow')
      expect(wrapper.find('[data-testid="anchor-preview"]').text())
        .toContain('project:vega task:retry-policy-main-workflow')
    })

    /**
     * The project is half of the task's anchor, so it is a field and not something the screen
     * decided quietly. It used to be inferred from the scope, which meant a task created from
     * All work landed in whichever project sorted first with nothing on screen saying so.
     */
    it('shows the project a task will land in, and lets it be changed', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()

      const parent = wrapper.find('[data-testid="record-parent"] select')
      expect(parent.exists()).toBe(true)
      expect((parent.element as HTMLSelectElement).value).toBe(vega)
      expect(parent.text()).toContain('project:vega')

      // The settled half of the anchor is on screen before a single character is typed.
      expect(wrapper.find('[data-testid="anchor-preview"]').text()).toContain('project:vega')
    })

    it('creates the task with both names and the project it landed in', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()
      await wrapper.find('[data-testid="record-title"]').setValue('Retry policy')
      await wrapper.find('[data-testid="record-save"]').trigger('click')
      await flushPromises()

      expect(createTask).toHaveBeenCalledWith({
        label: 'retry-policy',
        title: 'Retry policy',
        status: 'TODO',
        description: null,
        projectId: vega
      })
      expect(wrapper.find('[data-testid="record-dialog"]').exists()).toBe(false)
    })

    /** A hand-written label is a decision, and the title must not overwrite it afterwards. */
    it('stops following the title once the label is typed into', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()
      await wrapper.find('[data-testid="record-label"]').setValue('rp')
      await wrapper.find('[data-testid="record-title"]').setValue('Retry policy')
      await flushPromises()

      expect((wrapper.find('[data-testid="record-label"]').element as HTMLInputElement).value)
        .toBe('rp')
    })

    it('opens the selected task for editing on E, with its values loaded', async () => {
      const wrapper = await mountConsole()

      await wrapper.findAll('[data-testid="task-row"]')[0]!.trigger('click')
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'e' }))
      await flushPromises()

      expect((wrapper.find('[data-testid="record-title"]').element as HTMLInputElement).value)
        .toBe('Report builder')
      expect((wrapper.find('[data-testid="record-label"]').element as HTMLInputElement).value)
        .toBe('report-builder')
    })

    /**
     * The anchor is written down outside this application, in slash commands and in notes.
     * Moving it is allowed and has to be said out loud.
     */
    it('warns that the anchor moves when the label is changed on an existing record', async () => {
      const wrapper = await mountConsole()

      await wrapper.findAll('[data-testid="task-row"]')[0]!.trigger('click')
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'e' }))
      await flushPromises()

      expect(wrapper.find('[data-testid="rename-warning"]').exists()).toBe(false)

      await wrapper.find('[data-testid="record-label"]').setValue('validator')
      await flushPromises()

      expect(wrapper.find('[data-testid="rename-warning"]').text()).toContain('task:report-builder')
    })

    it('renames a title without touching the label the anchor uses', async () => {
      const wrapper = await mountConsole()

      await wrapper.find('[data-testid="scope-trigger"]').trigger('click')
      await wrapper.find('[data-testid="edit-project"]').trigger('click')
      await flushPromises()

      await wrapper.find('[data-testid="record-title"]').setValue('Vega, rebuilt')
      await wrapper.find('[data-testid="record-save"]').trigger('click')
      await flushPromises()

      expect(updateProject).toHaveBeenCalledWith(
        vega,
        expect.objectContaining({ label: 'vega', title: 'Vega, rebuilt' })
      )
    })

    /** The same field that decides where a task lands is what moves it afterwards. */
    it('moves a task to another project, anchor and all', async () => {
      const wrapper = await mountConsole()

      await wrapper.findAll('[data-testid="task-row"]')[0]!.trigger('click')
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'e' }))
      await flushPromises()

      await wrapper.find('[data-testid="record-parent"] select').setValue(beacon)
      await flushPromises()

      expect(wrapper.find('[data-testid="anchor-preview"]').text())
        .toContain('project:beacon task:report-builder')

      await wrapper.find('[data-testid="record-save"]').trigger('click')
      await flushPromises()

      expect(updateTask).toHaveBeenCalledWith(
        validator,
        expect.objectContaining({ label: 'report-builder', projectId: beacon })
      )
    })

    /** Nothing is deleted without the number of things going with it on screen first. */
    it('states the blast radius before deleting, and deletes once confirmed', async () => {
      const wrapper = await mountConsole()

      await wrapper.findAll('[data-testid="task-row"]')[0]!.trigger('click')
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'e' }))
      await flushPromises()

      await wrapper.find('[data-testid="record-delete"]').trigger('click')
      await flushPromises()

      expect(wrapper.text()).toContain('1 note unlinked')
      expect(deleteTask).not.toHaveBeenCalled()

      const confirm = wrapper
        .findAll('button')
        .find((button) => button.text() === 'Delete task')!
      await confirm.trigger('click')
      await flushPromises()

      expect(deleteTask).toHaveBeenCalledWith(validator)
    })

    it('refuses to save a record with no title', async () => {
      const wrapper = await mountConsole()

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 't' }))
      await flushPromises()
      await wrapper.find('[data-testid="record-save"]').trigger('click')
      await flushPromises()

      expect(createTask).not.toHaveBeenCalled()
      expect(wrapper.find('[data-testid="record-dialog"]').exists()).toBe(true)
    })
  })
})
