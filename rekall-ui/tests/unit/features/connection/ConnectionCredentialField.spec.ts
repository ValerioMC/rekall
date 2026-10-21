import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/connection/api/connections.api'
import ConnectionCredentialField from '@/features/connection/components/ConnectionCredentialField.vue'
import type { Connection } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

const GITLAB: Connection = {
  id: 'gitlab' as ConnectionId,
  companyId: 'c-1' as CompanyId,
  kind: 'GITLAB',
  label: 'gitlab',
  title: 'GitLab',
  baseUrl: 'https://gitlab.example.com',
  kubeContext: null,
  credentialSetAt: null,
  projectCount: 0,
  updatedAt: '2026-10-21T10:00:00Z'
}

describe('ConnectionCredentialField', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('sends a typed token once and keeps nothing of it afterwards', async () => {
    vi.mocked(api.setCredential).mockResolvedValue({ ...GITLAB, credentialSetAt: '2026-10-21T10:01:00Z' })
    const wrapper = mount(ConnectionCredentialField, { props: { connection: GITLAB } })
    expect(wrapper.find('[data-testid="credential-missing"]').exists()).toBe(true)

    await wrapper.get('[data-testid="replace-credential"]').trigger('click')
    await wrapper.get('[data-testid="gitlab-token"]').setValue('glpat-secret')
    await wrapper.get('[data-testid="save-credential"]').trigger('click')
    await flushPromises()

    expect(api.setCredential).toHaveBeenCalledWith('gitlab', { kind: 'GITLAB', token: 'glpat-secret' })
    expect(wrapper.find('[data-testid="gitlab-token"]').exists()).toBe(false)
    expect(wrapper.html()).not.toContain('glpat-secret')
  })

  it('drops the typed token when the store refuses it too', async () => {
    vi.mocked(api.setCredential).mockRejectedValue(new Error('refused'))
    const wrapper = mount(ConnectionCredentialField, { props: { connection: GITLAB } })

    await wrapper.get('[data-testid="replace-credential"]').trigger('click')
    await wrapper.get('[data-testid="gitlab-token"]').setValue('glpat-secret')
    await wrapper.get('[data-testid="save-credential"]').trigger('click')
    await flushPromises()

    expect((wrapper.get('[data-testid="gitlab-token"]').element as HTMLInputElement).value).toBe('')
  })
})
