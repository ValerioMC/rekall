import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import ProjectChecksEditor from '@/features/check/components/ProjectChecksEditor.vue'
import * as api from '@/features/check/api/checks.api'
import { PROJECT, check } from './fixtures'
import type { Project } from '@/features/catalog/model/catalog'

vi.mock('@/features/check/api/checks.api')

const project = (repoFolder: string | null): Project => ({ id: PROJECT, repoFolder }) as unknown as Project

async function render(repoFolder: string | null = '/work/vega') {
  const wrapper = mount(ProjectChecksEditor, { props: { project: project(repoFolder) } })
  await flushPromises()
  return wrapper
}

describe('the project checks editor', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchChecks).mockResolvedValue([])
  })

  it('says what a project with no checks does', async () => {
    const wrapper = await render()

    expect(wrapper.get('[data-testid="check-empty"]').text()).toContain('accepted as soon as the session makes it')
    expect(wrapper.find('[data-testid="checks-need-folder"]').exists()).toBe(false)
  })

  it('adds a check from the four fields', async () => {
    vi.mocked(api.createProjectCheck).mockResolvedValue(check('a', 0, null, 'backend tests'))
    const wrapper = await render()

    await wrapper.get('[data-testid="check-add"]').trigger('click')
    expect((wrapper.get('[data-testid="check-save"]').element as HTMLButtonElement).disabled).toBe(true)
    await wrapper.get('[data-testid="check-label-field"]').setValue('backend tests')
    await wrapper.get('[data-testid="check-command-field"]').setValue('cargo test --workspace')
    await wrapper.get('[data-testid="check-folder-field"]').setValue('rekall-ui')
    await wrapper.get('[data-testid="check-form"]').trigger('submit')
    await flushPromises()

    expect(api.createProjectCheck).toHaveBeenCalledWith(PROJECT, {
      label: 'backend tests',
      command: 'cargo test --workspace',
      folder: 'rekall-ui',
      timeoutSeconds: 600
    })
    expect(wrapper.findAll('[data-testid="check-row"]')).toHaveLength(1)
    expect(wrapper.find('[data-testid="check-form"]').exists()).toBe(false)
  })

  it('refuses an absolute folder and a timeout out of range before sending', async () => {
    const wrapper = await render()

    await wrapper.get('[data-testid="check-add"]').trigger('click')
    await wrapper.get('[data-testid="check-label-field"]').setValue('x')
    await wrapper.get('[data-testid="check-command-field"]').setValue('true')
    await wrapper.get('[data-testid="check-folder-field"]').setValue('/etc')
    expect((wrapper.get('[data-testid="check-save"]').element as HTMLButtonElement).disabled).toBe(true)
    await wrapper.get('[data-testid="check-folder-field"]').setValue('../up')
    expect((wrapper.get('[data-testid="check-save"]').element as HTMLButtonElement).disabled).toBe(true)
    await wrapper.get('[data-testid="check-folder-field"]').setValue('')
    await wrapper.get('[data-testid="check-timeout-field"]').setValue('0')
    expect((wrapper.get('[data-testid="check-save"]').element as HTMLButtonElement).disabled).toBe(true)
  })

  it('deletes a check from its row', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([check('a', 0, null, 'backend tests'), check('b', 1, null, 'ui lint')])
    vi.mocked(api.deleteCheck).mockResolvedValue(undefined)
    const wrapper = await render()

    await wrapper.findAll('[data-testid="check-delete"]')[0]!.trigger('click')
    await flushPromises()

    expect(api.deleteCheck).toHaveBeenCalledWith('a')
    expect(wrapper.findAll('[data-testid="check-row-label"]').map((label) => label.text())).toEqual(['ui lint'])
  })

  it('edits a check in place', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([check('a', 0, null, 'backend tests')])
    vi.mocked(api.updateCheck).mockResolvedValue(check('a', 0, null, 'unit tests'))
    const wrapper = await render()

    await wrapper.get('[data-testid="check-edit"]').trigger('click')
    await wrapper.get('[data-testid="check-label-field"]').setValue('unit tests')
    await wrapper.get('[data-testid="check-form"]').trigger('submit')
    await flushPromises()

    expect(api.updateCheck).toHaveBeenCalledWith('a', expect.objectContaining({ label: 'unit tests', timeoutSeconds: 600 }))
    expect(wrapper.get('[data-testid="check-row-label"]').text()).toBe('unit tests')
  })

  it('warns that a project with checks and no folder refuses every claim', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([check('a', 0)])

    const wrapper = await render(null)

    expect(wrapper.get('[data-testid="checks-need-folder"]').text()).toContain('Set the project')
  })

  it('moves a check down', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([check('a', 0), check('b', 1)])
    vi.mocked(api.reorderProjectChecks).mockResolvedValue([check('b', 0), check('a', 1)])
    const wrapper = await render()

    await wrapper.findAll('[data-testid="check-down"]')[0]!.trigger('click')
    await flushPromises()

    expect(api.reorderProjectChecks).toHaveBeenCalledWith(PROJECT, ['b', 'a'])
    expect(wrapper.findAll('[data-testid="check-row-label"]').map((label) => label.text())).toEqual(['b', 'a'])
  })
})
