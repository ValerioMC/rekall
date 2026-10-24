import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as catalogApi from '@/features/catalog/api/catalog.api'
import * as api from '@/features/connection/api/connections.api'
import JiraImportDialog from '@/features/connection/components/JiraImportDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { JiraIssue, ProjectConnection } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectConnectionId, ProjectId, TaskId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')
vi.mock('@/features/catalog/api/catalog.api')

const BINDING: ProjectConnection = {
  id: 'b-jira' as ProjectConnectionId,
  connectionId: 'jira' as ConnectionId,
  projectId: 'p-1' as ProjectId,
  kind: 'JIRA',
  label: 'jira',
  title: 'Acme Jira',
  baseUrl: 'https://example.atlassian.net',
  kubeContext: null,
  target: 'ENG',
  credentialSetAt: '2026-10-21T10:00:00Z'
}

function issue(key: string, taskId: string | null): JiraIssue {
  return {
    key,
    summary: `Summary of ${key}`,
    status: 'To Do',
    issueType: 'Story',
    url: `https://example.atlassian.net/browse/${key}`,
    taskId: taskId as TaskId | null
  }
}

function mountDialog() {
  return mount(JiraImportDialog, { props: { binding: BINDING }, attachTo: document.body })
}

describe('JiraImportDialog', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(catalogApi.fetchProjects).mockResolvedValue([])
    vi.mocked(catalogApi.fetchCompanies).mockResolvedValue([])
  })

  it('lists the open issues and lets only the ones not yet imported be picked', async () => {
    vi.mocked(api.fetchJiraIssues).mockResolvedValue([issue('ENG-1', 't-1'), issue('ENG-2', null)])
    const wrapper = mountDialog()
    await flushPromises()

    expect(api.fetchJiraIssues).toHaveBeenCalledWith('p-1', 'b-jira')
    expect(wrapper.findAll('[data-testid="jira-issue"]')).toHaveLength(2)
    expect(wrapper.findAll('[data-testid="jira-issue-imported"]')).toHaveLength(1)
    expect(wrapper.find('[data-testid="pick-ENG-1"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="import-chosen-issues"]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('imports the picked issues into the console as tasks and closes', async () => {
    const created = { id: 't-2' as TaskId, label: 'eng-2', projectId: 'p-1' } as Task
    vi.mocked(api.fetchJiraIssues).mockResolvedValue([issue('ENG-2', null), issue('ENG-3', null)])
    vi.mocked(api.importJiraIssues).mockResolvedValue([created])
    const wrapper = mountDialog()
    await flushPromises()

    await wrapper.get('[data-testid="pick-ENG-2"] input').setValue(true)
    await wrapper.get('[data-testid="import-chosen-issues"]').trigger('click')
    await flushPromises()

    expect(api.importJiraIssues).toHaveBeenCalledWith('p-1', 'b-jira', ['ENG-2'])
    expect(useConsoleStore().tasks.map((task) => task.id)).toEqual(['t-2'])
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })

  it('says when Jira could not be read', async () => {
    vi.mocked(api.fetchJiraIssues).mockRejectedValue(new Error('Jira rejected the stored credential (HTTP 401).'))
    const wrapper = mountDialog()
    await flushPromises()

    expect(wrapper.find('[data-testid="jira-issues-failed"]').exists()).toBe(true)
    wrapper.unmount()
  })
})
