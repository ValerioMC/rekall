import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/connection/api/connections.api'
import ConnectionStoredCredential from '@/features/connection/components/ConnectionStoredCredential.vue'
import type { Connection } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId } from '@/model/branded'

vi.mock('@/features/connection/api/connections.api')

const GITLAB: Connection = {
  id: 'gitlab' as ConnectionId,
  companyIds: ['c-1' as CompanyId],
  rootPath: null,
  kind: 'GITLAB',
  label: 'gitlab',
  title: 'GitLab',
  baseUrl: 'https://gitlab.example.com',
  kubeContext: null,
  proxyUrl: null,
  acceptSelfSigned: false,
  credentialSetAt: '2026-10-21T10:01:00Z',
  projectCount: 0,
  updatedAt: '2026-10-21T10:01:00Z'
}

const KUBERNETES: Connection = { ...GITLAB, id: 'cluster' as ConnectionId, kind: 'KUBERNETES' }
const JIRA: Connection = { ...GITLAB, id: 'jira' as ConnectionId, kind: 'JIRA' }

function field(wrapper: ReturnType<typeof mount>): HTMLInputElement {
  return wrapper.get('[data-testid="stored-credential-secret"]').element as HTMLInputElement
}

describe('ConnectionStoredCredential', () => {
  const writeText = vi.fn<(text: string) => Promise<void>>()

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    writeText.mockResolvedValue()
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true })
  })

  it('hides the credential and asks the server for nothing until the eye is pressed', () => {
    const wrapper = mount(ConnectionStoredCredential, { props: { connection: GITLAB } })

    expect(field(wrapper).type).toBe('password')
    expect(api.revealCredential).not.toHaveBeenCalled()
  })

  it('shows the stored token when the eye is pressed and hides it again with the second press', async () => {
    vi.mocked(api.revealCredential).mockResolvedValue({ token: 'glpat-secret', kubeconfig: null, email: null })
    const wrapper = mount(ConnectionStoredCredential, { props: { connection: GITLAB } })

    await wrapper.get('[data-testid="toggle-stored-credential"]').trigger('click')
    await flushPromises()
    expect(field(wrapper).type).toBe('text')
    expect(field(wrapper).value).toBe('glpat-secret')

    await wrapper.get('[data-testid="toggle-stored-credential"]').trigger('click')
    expect(field(wrapper).type).toBe('password')
    expect(wrapper.html()).not.toContain('glpat-secret')
  })

  it('copies the stored token without showing it', async () => {
    vi.mocked(api.revealCredential).mockResolvedValue({ token: 'glpat-secret', kubeconfig: null, email: null })
    const wrapper = mount(ConnectionStoredCredential, { props: { connection: GITLAB } })

    await wrapper.get('[data-testid="copy-stored-credential"]').trigger('click')
    await flushPromises()

    expect(writeText).toHaveBeenCalledWith('glpat-secret')
    expect(field(wrapper).type).toBe('password')
  })

  it('shows a stored kubeconfig on several lines', async () => {
    vi.mocked(api.revealCredential).mockResolvedValue({ token: null, kubeconfig: 'apiVersion: v1\nkind: Config', email: null })
    const wrapper = mount(ConnectionStoredCredential, { props: { connection: KUBERNETES } })

    await wrapper.get('[data-testid="toggle-stored-credential"]').trigger('click')
    await flushPromises()

    const shown = wrapper.get('[data-testid="stored-credential-secret"]').element as HTMLTextAreaElement
    expect(shown.tagName).toBe('TEXTAREA')
    expect(shown.value).toBe('apiVersion: v1\nkind: Config')
  })

  it('shows and copies the email of a Jira Cloud credential beside its token', async () => {
    vi.mocked(api.revealCredential).mockResolvedValue({ token: 'api-token', kubeconfig: null, email: 'me@example.com' })
    const wrapper = mount(ConnectionStoredCredential, { props: { connection: JIRA } })

    await wrapper.get('[data-testid="toggle-stored-credential"]').trigger('click')
    await flushPromises()
    await wrapper.get('[data-testid="copy-stored-email"]').trigger('click')
    await flushPromises()

    expect((wrapper.get('[data-testid="stored-credential-email"]').element as HTMLInputElement).value).toBe('me@example.com')
    expect(writeText).toHaveBeenCalledWith('me@example.com')
  })

  it('drops the shown credential when it is replaced', async () => {
    vi.mocked(api.revealCredential).mockResolvedValue({ token: 'glpat-secret', kubeconfig: null, email: null })
    const wrapper = mount(ConnectionStoredCredential, { props: { connection: GITLAB } })
    await wrapper.get('[data-testid="toggle-stored-credential"]').trigger('click')
    await flushPromises()

    await wrapper.setProps({ connection: { ...GITLAB, credentialSetAt: '2026-10-21T11:00:00Z' } })

    expect(field(wrapper).type).toBe('password')
    expect(wrapper.html()).not.toContain('glpat-secret')
  })

  describe('editing the shown value', () => {
    async function shown(connection: Connection, credential: { token: string | null; kubeconfig: string | null; email: string | null }) {
      vi.mocked(api.revealCredential).mockResolvedValue(credential)
      const wrapper = mount(ConnectionStoredCredential, { props: { connection } })
      await wrapper.get('[data-testid="toggle-stored-credential"]').trigger('click')
      await flushPromises()
      return wrapper
    }

    it('offers no save until the shown value is changed', async () => {
      const wrapper = await shown(GITLAB, { token: 'glpat-secret', kubeconfig: null, email: null })

      expect(wrapper.find('[data-testid="save-stored-credential"]').exists()).toBe(false)

      await wrapper.get('[data-testid="stored-credential-secret"]').setValue('glpat-new')
      expect(wrapper.find('[data-testid="save-stored-credential"]').exists()).toBe(true)
    })

    it('stores the modified token', async () => {
      vi.mocked(api.setCredential).mockResolvedValue({ ...GITLAB, credentialSetAt: '2026-10-21T11:00:00Z' })
      const wrapper = await shown(GITLAB, { token: 'glpat-secret', kubeconfig: null, email: null })

      await wrapper.get('[data-testid="stored-credential-secret"]').setValue('glpat-new')
      await wrapper.get('[data-testid="save-stored-credential"]').trigger('click')
      await flushPromises()

      expect(api.setCredential).toHaveBeenCalledWith(GITLAB.id, { kind: 'GITLAB', token: 'glpat-new' })
    })

    it('stores a modified Jira Cloud email with its token', async () => {
      vi.mocked(api.setCredential).mockResolvedValue({ ...JIRA, credentialSetAt: '2026-10-21T11:00:00Z' })
      const wrapper = await shown(JIRA, { token: 'api-token', kubeconfig: null, email: 'me@example.com' })

      await wrapper.get('[data-testid="stored-credential-email"]').setValue('you@example.com')
      await wrapper.get('[data-testid="save-stored-credential"]').trigger('click')
      await flushPromises()

      expect(api.setCredential).toHaveBeenCalledWith(JIRA.id, {
        kind: 'JIRA',
        deployment: 'CLOUD',
        email: 'you@example.com',
        token: 'api-token'
      })
    })

    it('puts the stored value back on cancel', async () => {
      const wrapper = await shown(GITLAB, { token: 'glpat-secret', kubeconfig: null, email: null })

      await wrapper.get('[data-testid="stored-credential-secret"]').setValue('glpat-new')
      await wrapper.get('[data-testid="discard-stored-credential"]').trigger('click')

      expect(field(wrapper).value).toBe('glpat-secret')
      expect(wrapper.find('[data-testid="save-stored-credential"]').exists()).toBe(false)
    })

    it('does not let an emptied token be stored', async () => {
      const wrapper = await shown(GITLAB, { token: 'glpat-secret', kubeconfig: null, email: null })

      await wrapper.get('[data-testid="stored-credential-secret"]').setValue('')

      expect(wrapper.get('[data-testid="save-stored-credential"]').attributes('disabled')).toBeDefined()
    })
  })
})
