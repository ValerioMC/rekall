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
    const gitlab = mountBinder('GITLAB', true)
    await typeTarget(gitlab, 'group/p')

    expect(api.fetchTargetSuggestions).not.toHaveBeenCalled()
    expect(withoutCredential.find('datalist').exists()).toBe(false)
    withoutCredential.unmount()
    gitlab.unmount()
  })
})
