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

  it('warns about a connection without a credential and links each to its company screen', () => {
    const wrapper = mountChips([PROD, STAGING])

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
