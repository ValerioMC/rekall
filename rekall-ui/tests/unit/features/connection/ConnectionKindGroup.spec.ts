import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import ConnectionKindGroup from '@/features/connection/components/ConnectionKindGroup.vue'
import type { Connection, ConnectionKind } from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'
import { GLOBEX, aConnection } from '../../support/connectionFixture'

const KUBERNETES: ConnectionKind = 'KUBERNETES'

async function mountGroup(kind: ConnectionKind, connections: readonly Connection[], selectedId: string | null = null) {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/connections/:id?', name: 'connections', component: { template: '<div />' } }]
  })
  await router.push('/connections')
  return mount(ConnectionKindGroup, {
    props: { kind, connections, selectedId: selectedId as ConnectionId | null },
    global: { plugins: [router] }
  })
}

describe('ConnectionKindGroup', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it("lists its connections under the kind's logo and name, with how many there are", async () => {
    const wrapper = await mountGroup(KUBERNETES, [
      aConnection('kmaster14', { kind: KUBERNETES, baseUrl: null, kubeContext: 'kmaster14' }),
      aConnection('staging', { kind: KUBERNETES, baseUrl: null, kubeContext: 'staging' })
    ])

    expect(wrapper.find('svg[data-kind="KUBERNETES"]').exists()).toBe(true)
    expect(wrapper.get('[data-testid="connection-group-name"]').text()).toBe('Kubernetes')
    expect(wrapper.get('[data-testid="connection-group-count"]').text()).toBe('(2)')
    expect(wrapper.findAll('[data-testid="connection-row"]')).toHaveLength(2)
  })

  it('marks the selected row as current and links every row to its own detail', async () => {
    const wrapper = await mountGroup(
      'GITLAB',
      [aConnection('platform'), aConnection('clients')],
      'clients'
    )
    const rows = wrapper.findAll('[data-testid="connection-row"]')

    expect(rows.map((row) => row.attributes('href'))).toEqual(['/connections/platform', '/connections/clients'])
    expect(rows.map((row) => row.attributes('aria-current'))).toEqual([undefined, 'true'])
  })

  it('says what a row covers, whether it has a credential and how widely it is shared', async () => {
    const wrapper = await mountGroup('GITLAB', [
      aConnection('acme', { rootPath: 'acme', credentialSetAt: null, companyIds: [], projectCount: 3 }),
      aConnection('shared', { companyIds: [GLOBEX, ...aConnection('x').companyIds], projectCount: 1 })
    ])
    const [unshared, shared] = wrapper.findAll('[data-testid="connection-row"]')

    expect(unshared?.get('[data-testid="connection-scope"]').text()).toBe('gitlab.example.com/acme')
    expect(unshared?.text()).toContain('No credential')
    expect(unshared?.text()).toContain('Not shared with any company')
    expect(unshared?.get('[data-testid="connection-sharing"]').text()).toBe('3 projects')
    expect(shared?.text()).toContain('Stored')
    expect(shared?.find('[data-testid="connection-unshared"]').exists()).toBe(false)
    expect(shared?.get('[data-testid="connection-sharing"]').text()).toBe('2 companies · 1 project')
  })
})
