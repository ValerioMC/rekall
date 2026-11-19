import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import * as api from '@/features/connection/api/connections.api'
import ProjectTaskManagerPane from '@/features/project/components/ProjectTaskManagerPane.vue'
import type { Project } from '@/features/catalog/model/catalog'
import type { ConnectionKind, ProjectConnection } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectConnectionId, ProjectId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

const PROJECT = { id: 'p-1' as ProjectId, title: 'Vega' } as unknown as Project

function binding(label: string, kind: ConnectionKind, over: Partial<ProjectConnection> = {}): ProjectConnection {
  return {
    id: `b-${label}` as ProjectConnectionId,
    connectionId: label as ConnectionId,
    projectId: PROJECT.id,
    kind,
    label,
    title: `The ${label}`,
    baseUrl: null,
    kubeContext: null,
    target: 'acme/vega',
    taskManager: false,
    credentialSetAt: '2026-10-21T10:00:00Z',
    ...over
  }
}

async function render(bindings: ProjectConnection[]) {
  vi.mocked(api.fetchProjectConnections).mockResolvedValue(bindings)
  const wrapper = mount(ProjectTaskManagerPane, { props: { project: PROJECT } })
  await flushPromises()
  return wrapper
}

describe('the task manager pane', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('offers only the GitLab, GitHub and Jira bindings, and none is the choice while no binding is the task manager', async () => {
    const wrapper = await render([binding('prod', 'KUBERNETES'), binding('gitlab', 'GITLAB'), binding('jira', 'JIRA')])

    expect(wrapper.find('[data-testid="task-manager-prod"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="task-manager-gitlab"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="task-manager-jira"]').exists()).toBe(true)
    expect((wrapper.get('[data-testid="task-manager-none"]').element as HTMLInputElement).checked).toBe(true)
  })

  it('says so when the project uses none of them', async () => {
    const wrapper = await render([binding('prod', 'KUBERNETES')])

    expect(wrapper.find('[data-testid="task-manager-empty"]').exists()).toBe(true)
  })

  it('makes the picked binding the task manager and keeps its label and target', async () => {
    const gitlab = binding('gitlab', 'GITLAB')
    vi.mocked(api.updateBinding).mockResolvedValue({ ...gitlab, taskManager: true })
    const wrapper = await render([gitlab])

    await wrapper.get('[data-testid="task-manager-gitlab"]').setValue(true)
    await flushPromises()

    expect(api.updateBinding).toHaveBeenCalledWith('p-1', 'b-gitlab', { label: 'gitlab', target: 'acme/vega', taskManager: true })
  })

  it('releases the task manager when none is picked', async () => {
    const github = binding('github', 'GITHUB', { taskManager: true })
    vi.mocked(api.updateBinding).mockResolvedValue({ ...github, taskManager: false })
    const wrapper = await render([github])
    expect((wrapper.get('[data-testid="task-manager-github"]').element as HTMLInputElement).checked).toBe(true)

    await wrapper.get('[data-testid="task-manager-none"]').setValue(true)
    await flushPromises()

    expect(api.updateBinding).toHaveBeenCalledWith('p-1', 'b-github', { label: 'github', target: 'acme/vega', taskManager: false })
  })

  it('cannot pick a binding that names no place for the issues', async () => {
    const wrapper = await render([binding('jira', 'JIRA', { target: null })])

    expect((wrapper.get('[data-testid="task-manager-jira"]').element as HTMLInputElement).disabled).toBe(true)
  })
})
