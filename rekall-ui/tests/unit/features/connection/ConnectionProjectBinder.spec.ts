import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/connection/api/connections.api'
import ConnectionProjectBinder from '@/features/connection/components/ConnectionProjectBinder.vue'
import type { ConnectionKind } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

function mountBinder(kind: ConnectionKind, credentialSet: boolean) {
  return mount(ConnectionProjectBinder, {
    props: {
      connectionId: 'site' as ConnectionId,
      connectionTitle: 'The site',
      kind,
      credentialSet,
      baseUrl: 'https://gitlab.acme.dev',
      rootPath: 'platform',
      projects: [{ id: 'p-1' as ProjectId, title: 'Vega' }]
    },
    attachTo: document.body
  })
}

async function typeTarget(wrapper: ReturnType<typeof mountBinder>, text: string): Promise<void> {
  await wrapper.get('[data-testid="binder-target"]').setValue(text)
  await new Promise((resolve) => setTimeout(resolve, 300))
  await flushPromises()
}

describe('ConnectionProjectBinder target suggestions', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('offers the projects of a Jira site that has a credential while the key is typed', async () => {
    vi.mocked(api.fetchTargetSuggestions).mockResolvedValue([{ value: 'ENG', name: 'Engineering' }])
    const wrapper = mountBinder('JIRA', true)

    await typeTarget(wrapper, 'en')

    expect(api.fetchTargetSuggestions).toHaveBeenCalledWith('site', 'en')
    expect(wrapper.findAll('datalist option').map((option) => option.attributes('value'))).toEqual(['ENG'])
    wrapper.unmount()
  })

  it('says why the site could not suggest in the field hint', async () => {
    vi.mocked(api.fetchTargetSuggestions).mockRejectedValue(new Error('Jira could not be asked: it answered HTTP 500.'))
    const wrapper = mountBinder('JIRA', true)

    await typeTarget(wrapper, 'en')

    expect(wrapper.text()).toContain('Suggestions unavailable: Jira could not be asked')
    wrapper.unmount()
  })

  it('asks nothing of a site without a credential or of a kind that has no list', async () => {
    const withoutCredential = mountBinder('CONFLUENCE', false)
    await typeTarget(withoutCredential, 'do')
    const kubernetes = mountBinder('KUBERNETES', true)
    await typeTarget(kubernetes, 'apps')

    expect(api.fetchTargetSuggestions).not.toHaveBeenCalled()
    expect(withoutCredential.find('datalist').exists()).toBe(false)
    withoutCredential.unmount()
    kubernetes.unmount()
  })
})

const BILLING = {
  kind: 'PROJECT',
  path: 'platform/backend/billing',
  name: 'Billing',
  visibility: 'private',
  defaultBranch: 'main',
  lastActivityAt: null,
  subgroupCount: null,
  projectCount: null
} as const

async function chooseProject(wrapper: ReturnType<typeof mountBinder>): Promise<void> {
  await wrapper.get('select').setValue('p-1')
}

describe('ConnectionProjectBinder GitLab path', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchGitlabChildren).mockResolvedValue([BILLING])
  })

  it('binds the whole connection while the path is empty', async () => {
    const wrapper = mountBinder('GITLAB', true)
    await chooseProject(wrapper)

    expect(wrapper.get('[data-testid="binder-submit"]').attributes('disabled')).toBeUndefined()
    expect(wrapper.get('[data-testid="target-verdict"]').attributes('data-verdict')).toBe('BLANK')
    wrapper.unmount()
  })

  it('holds the bind until GitLab has the typed path, then binds the path it found', async () => {
    vi.mocked(api.fetchTargetCheck).mockResolvedValue({ status: 'FOUND', found: BILLING })
    vi.mocked(api.bindConnection).mockResolvedValue({} as never)
    const wrapper = mountBinder('GITLAB', true)
    await chooseProject(wrapper)

    await wrapper.get('[data-testid="binder-target"]').setValue('https://gitlab.acme.dev/platform/backend/billing.git')
    expect(wrapper.get('[data-testid="binder-submit"]').attributes('disabled')).toBeDefined()
    await new Promise((resolve) => setTimeout(resolve, 300))
    await flushPromises()
    await wrapper.get('[data-testid="binder-target"]').trigger('blur')

    expect(api.fetchTargetCheck).toHaveBeenCalledWith('site', 'platform/backend/billing')
    expect(wrapper.get('[data-testid="target-verdict"]').attributes('data-verdict')).toBe('FOUND')
    expect(wrapper.get('[data-testid="binder-submit"]').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it('refuses a path GitLab does not have and offers the nearest one', async () => {
    vi.mocked(api.fetchTargetCheck).mockResolvedValue({ status: 'NOT_FOUND', nearest: 'platform/backend/billing' })
    const wrapper = mountBinder('GITLAB', true)
    await chooseProject(wrapper)

    await typeTarget(wrapper, 'platform/backend/bilng')
    await wrapper.get('[data-testid="binder-target"]').trigger('blur')

    expect(wrapper.get('[data-testid="target-verdict"]').attributes('data-verdict')).toBe('NOT_FOUND')
    expect(wrapper.get('[data-testid="binder-submit"]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-testid="use-nearest"]').text()).toContain('platform/backend/billing')
    wrapper.unmount()
  })

  it('lists what sits under the group typed so far and completes the path from a row', async () => {
    vi.mocked(api.fetchTargetCheck).mockResolvedValue({ status: 'FOUND', found: BILLING })
    const wrapper = mountBinder('GITLAB', true)
    await chooseProject(wrapper)

    await wrapper.get('[data-testid="binder-target"]').trigger('focus')
    await typeTarget(wrapper, 'platform/backend/')
    expect(api.fetchGitlabChildren).toHaveBeenLastCalledWith('site', 'platform/backend', '')
    await wrapper.get('[data-testid="gitlab-path-option"]').trigger('click')
    await flushPromises()

    expect((wrapper.get('[data-testid="binder-target"]').element as HTMLInputElement).value).toBe('platform/backend/billing')
    wrapper.unmount()
  })
})
