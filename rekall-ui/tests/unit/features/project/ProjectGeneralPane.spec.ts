import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import ProjectGeneralPane from '@/features/project/components/ProjectGeneralPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

const acme = { id: 'c-1' as CompanyId, name: 'Acme' } as Company
const globex = { id: 'c-2' as CompanyId, name: 'Globex' } as Company

const vega = {
  id: 'p-1' as ProjectId,
  companyId: acme.id,
  label: 'vega',
  title: 'Vega',
  description: 'Billing platform.',
  status: 'ACTIVE',
  icon: 'folder',
  repoFolder: '/work/vega',
  autoCommit: false,
  blueprintMarkdown: null
} as unknown as Project

let wrapper: VueWrapper | null = null
let updateProject: ReturnType<typeof vi.fn>
let deleteProject: ReturnType<typeof vi.fn>

function render(): VueWrapper {
  const store = useConsoleStore()
  store.companies = [acme, globex]
  store.projects = [vega]
  store.tasks = []
  wrapper = mount(ProjectGeneralPane, { props: { project: vega }, attachTo: document.body })
  return wrapper
}

beforeEach(() => {
  setActivePinia(createPinia())
  const store = useConsoleStore()
  updateProject = vi.fn().mockResolvedValue(undefined)
  deleteProject = vi.fn().mockResolvedValue(undefined)
  store.updateProject = updateProject as never
  store.deleteProject = deleteProject as never
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('the general pane', () => {
  it('shows the project as stored and offers no save until something changes', () => {
    const pane = render()

    expect((pane.get('[data-testid="record-title"]').element as HTMLInputElement).value).toBe('Vega')
    expect((pane.get('[data-testid="record-label"]').element as HTMLInputElement).value).toBe('vega')
    expect((pane.get('[data-testid="project-description"]').element as HTMLTextAreaElement).value).toBe('Billing platform.')
    expect(pane.find('[data-testid="settings-save-bar"]').exists()).toBe(false)
  })

  it('saves a rename and a move to another company in one write', async () => {
    const pane = render()

    await pane.get('[data-testid="record-title"]').setValue('Vega Platform')
    await pane.get('[data-testid="project-company"] select').setValue(globex.id)
    await pane.get('[data-testid="settings-save"]').trigger('click')
    await flushPromises()

    expect(updateProject).toHaveBeenCalledOnce()
    expect(updateProject).toHaveBeenCalledWith(
      'p-1',
      expect.objectContaining({
        label: 'vega',
        title: 'Vega Platform',
        companyId: 'c-2',
        repoFolder: '/work/vega',
        description: 'Billing platform.'
      })
    )
  })

  it('refuses a blank title and writes nothing', async () => {
    const pane = render()

    await pane.get('[data-testid="record-title"]').setValue('  ')
    await pane.get('[data-testid="settings-save"]').trigger('click')

    expect(updateProject).not.toHaveBeenCalled()
    expect(pane.text()).toContain('A title is required.')
  })

  it('puts the stored values back on revert', async () => {
    const pane = render()
    await pane.get('[data-testid="project-description"]').setValue('Something else')

    await pane.get('[data-testid="settings-revert"]').trigger('click')

    expect((pane.get('[data-testid="project-description"]').element as HTMLTextAreaElement).value).toBe('Billing platform.')
    expect(pane.find('[data-testid="settings-save-bar"]').exists()).toBe(false)
  })

  it('deletes only after the confirmation, then says so', async () => {
    const pane = render()

    await pane.get('[data-testid="project-delete"]').trigger('click')
    expect(deleteProject).not.toHaveBeenCalled()
    const confirm = Array.from(document.body.querySelectorAll('button')).find((button) => button.textContent?.trim() === 'Delete project' && button.closest('[role="alertdialog"]'))
    confirm?.click()
    await flushPromises()

    expect(deleteProject).toHaveBeenCalledWith('p-1')
    expect(pane.emitted('deleted')).toHaveLength(1)
  })
})
