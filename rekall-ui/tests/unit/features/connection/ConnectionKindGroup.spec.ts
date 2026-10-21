import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ConnectionKindGroup from '@/features/connection/components/ConnectionKindGroup.vue'
import type { Connection, ConnectionKind } from '@/features/connection/model/connection'
import type { CompanyId, ConnectionId } from '@/model/branded'

function connection(id: string, kind: ConnectionKind, credentialSetAt: string | null): Connection {
  return {
    id: id as ConnectionId,
    companyId: 'c-1' as CompanyId,
    kind,
    label: id,
    title: `The ${id}`,
    baseUrl: kind === 'GITLAB' ? 'https://gitlab.example.com' : 'https://10.0.0.14:6443',
    kubeContext: kind === 'KUBERNETES' ? id : null,
    credentialSetAt,
    projectCount: 1,
    updatedAt: '2026-10-21T10:00:00Z'
  }
}

const ALL = [
  connection('gitlab', 'GITLAB', '2026-10-21T10:00:00Z'),
  connection('kmaster14', 'KUBERNETES', null),
  connection('staging', 'KUBERNETES', '2026-10-21T10:00:00Z')
]

function mountGroup(kind: ConnectionKind, connections: readonly Connection[] = ALL) {
  return mount(ConnectionKindGroup, { props: { kind, connections, selectedId: 'staging' as ConnectionId } })
}

describe('ConnectionKindGroup', () => {
  it("lists only the connections of its kind under that kind's logo", () => {
    const wrapper = mountGroup('KUBERNETES')

    expect(wrapper.find('svg[data-kind="KUBERNETES"]').exists()).toBe(true)
    expect(wrapper.findAll('[data-testid="connection-row"]').map((row) => row.text())).toEqual([
      expect.stringContaining('The kmaster14'),
      expect.stringContaining('The staging')
    ])
    expect(wrapper.get('[data-testid="connection-group-count"]').text()).toBe('2')
    expect(wrapper.text()).toContain('1 without a credential')
  })

  it('marks the selected row and emits the one clicked', async () => {
    const wrapper = mountGroup('KUBERNETES')
    const rows = wrapper.findAll('[data-testid="connection-row"]')

    expect(rows[1]?.attributes('aria-current')).toBe('true')
    await rows[0]?.trigger('click')
    expect(wrapper.emitted('select')).toEqual([['kmaster14']])
  })

  it('shows where a GitLab connection points and asks to add one of its kind', async () => {
    const wrapper = mountGroup('GITLAB')

    expect(wrapper.get('[data-testid="connection-row"]').text()).toContain('gitlab.example.com')
    await wrapper.get('[data-testid="add-connection-GITLAB"]').trigger('click')
    expect(wrapper.emitted('add')).toHaveLength(1)
  })

  it('says so when the company has none of its kind', () => {
    const wrapper = mountGroup('GITLAB', ALL.slice(1))

    expect(wrapper.get('[data-testid="connection-group-empty"]').text()).toBe('No GitLab instance yet.')
  })
})
