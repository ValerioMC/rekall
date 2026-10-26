import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import TerminalConnectionChips from '@/features/connection/components/TerminalConnectionChips.vue'
import { useCredentialAccessStore } from '@/features/connection/store/credentialAccess.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { TerminalConnection } from '@/features/claude/model/terminal'
import type { CompanyId, ConnectionId, ProjectId, TerminalId } from '@/model/branded'

const TERMINAL = 't-1' as TerminalId
const PROJECT = 'p-1' as ProjectId
const PROD: TerminalConnection = {
  connectionId: 'prod' as ConnectionId,
  label: 'prod',
  kind: 'KUBERNETES',
  credentialSet: true
}
const STAGING: TerminalConnection = {
  connectionId: 'staging' as ConnectionId,
  label: 'staging',
  kind: 'KUBERNETES',
  credentialSet: false
}
const REPO: TerminalConnection = {
  connectionId: 'repo' as ConnectionId,
  label: 'api-repo',
  kind: 'GITLAB',
  credentialSet: true
}
const BOARD: TerminalConnection = {
  connectionId: 'board' as ConnectionId,
  label: 'board',
  kind: 'JIRA',
  credentialSet: true
}

function mountChips(connections: readonly TerminalConnection[]) {
  return mount(TerminalConnectionChips, {
    props: { terminalId: TERMINAL, projectId: PROJECT, connections },
    global: { stubs: { RouterLink: { props: ['to'], template: '<a :href="to"><slot /></a>' } } }
  })
}

describe('TerminalConnectionChips', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    useConsoleStore().projects = [{ id: PROJECT, companyId: 'c-1' as CompanyId } as unknown as Project]
  })

  it('shows nothing for a terminal whose project reaches nothing', () => {
    expect(mountChips([]).find('[data-testid="terminal-connections"]').exists()).toBe(false)
  })

  it('folds the connections never asked for into one unused chip', () => {
    const wrapper = mountChips([PROD, STAGING])

    expect(wrapper.find('[data-testid="terminal-connection-prod"]').exists()).toBe(false)
    const unused = wrapper.get('[data-testid="terminal-connections-unused"]')
    expect(unused.text()).toBe('2 unused')
    expect(unused.attributes('title')).toBe('prod, staging')
  })

  it('shows a connection on its own, with its count, from the first time it is asked for', async () => {
    const wrapper = mountChips([PROD, STAGING])

    useCredentialAccessStore().record({ terminalId: TERMINAL, connectionId: PROD.connectionId, granted: true })
    await wrapper.vm.$nextTick()

    const chip = wrapper.get('[data-testid="terminal-connection-prod"]')
    expect(chip.attributes('data-tone')).toBe('used')
    expect(chip.text()).toContain('·1')
    expect(wrapper.get('[data-testid="terminal-connections-unused"]').text()).toBe('1 unused')
  })

  it('orders the connections by kind, clusters first, then GitLab, then Jira', async () => {
    const wrapper = mountChips([BOARD, REPO, STAGING, PROD])

    await wrapper.get('[data-testid="terminal-connections-unused"]').trigger('click')

    const order = wrapper.findAll('[data-tone]').map((chip) => chip.attributes('data-testid'))
    expect(order).toEqual([
      'terminal-connection-prod',
      'terminal-connection-staging',
      'terminal-connection-api-repo',
      'terminal-connection-board'
    ])
  })

  it('warns about a connection without a credential and links each to its company screen', async () => {
    const wrapper = mountChips([PROD, STAGING])
    await wrapper.get('[data-testid="terminal-connections-unused"]').trigger('click')

    expect(wrapper.get('[data-testid="terminal-connection-prod"]').attributes('data-tone')).toBe('idle')
    const staging = wrapper.get('[data-testid="terminal-connection-staging"]')
    expect(staging.attributes('data-tone')).toBe('warn')
    expect(staging.attributes('title')).toContain('no credential')
    expect(staging.attributes('href')).toBe('/companies/c-1/connections')
  })

  it('counts each hand-out as it is announced and warns once one is refused', async () => {
    const wrapper = mountChips([PROD])
    const access = useCredentialAccessStore()

    access.record({ terminalId: TERMINAL, connectionId: PROD.connectionId, granted: true })
    access.record({ terminalId: TERMINAL, connectionId: PROD.connectionId, granted: true })
    access.record({ terminalId: 'other' as TerminalId, connectionId: PROD.connectionId, granted: true })
    await wrapper.vm.$nextTick()

    const chip = wrapper.get('[data-testid="terminal-connection-prod"]')
    expect(chip.attributes('data-tone')).toBe('used')
    expect(chip.text()).toContain('·2')

    access.record({ terminalId: TERMINAL, connectionId: PROD.connectionId, granted: false })
    await wrapper.vm.$nextTick()

    const refused = wrapper.get('[data-testid="terminal-connection-prod"]')
    expect(refused.attributes('data-tone')).toBe('warn')
    expect(refused.attributes('title')).toContain('refused 1 time')
  })
})
