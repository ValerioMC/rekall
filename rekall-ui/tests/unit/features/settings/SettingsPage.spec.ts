import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { installStorage } from '../../support/memoryStorage'
import { flushPromises } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import { preferredProvider } from '@/common/config/session-launch'
import SettingsPage from '@/features/settings/components/SettingsPage.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import { mountWithHeaderChrome } from '../../support/mountWithHeaderChrome'

vi.mock('@/features/settings/components/DatabaseSection.vue', () => ({ default: { template: '<div data-testid="database-section" />' } }))
vi.mock('@/features/settings/components/VersionSection.vue', () => ({ default: { template: '<div data-testid="version-section" />' } }))
vi.mock('@/features/settings/components/ClaudeCodeSection.vue', () => ({ default: { template: '<div data-testid="claude-section" />' } }))
vi.mock('@/features/settings/components/CodexSection.vue', () => ({ default: { template: '<div data-testid="codex-settings" />' } }))
vi.mock('@/features/settings/components/GuardProtectionSection.vue', () => ({ default: { template: '<div />' } }))
vi.mock('@/features/picture/components/PictureCleanupPanel.vue', () => ({ default: { template: '<section data-testid="picture-cleanup" />' } }))
vi.mock('@/features/settings/components/BackupsSection.vue', () => ({ default: { template: '<div data-testid="backups-section" />' } }))
vi.mock('@/features/settings/components/LangfuseSection.vue', () => ({ default: { template: '<div data-testid="langfuse-section" />' } }))

async function mountAt(path: string) {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/settings/:section?/:topic?', name: 'settings', component: { render: () => null } }, { path: '/:p(.*)*', component: { render: () => null } }] })
  await router.push(path)
  const page = mountWithHeaderChrome(SettingsPage, { global: { plugins: [router] } })
  await flushPromises()
  return page
}

describe('the Settings screen', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
  })

  it('opens on General and lists every section in the rail, grouped', async () => {
    const page = await mountAt('/settings')

    expect(page.find('[data-testid="database-section"]').exists()).toBe(true)
    expect(page.findAll('[data-testid="settings-rail"] > div > p').map((group) => group.text())).toEqual(['Workspace', 'Work', 'Console', 'Reference'])
    expect(page.findAll('[data-testid="settings-rail"] > div > div > a').map((link) => link.text())).toEqual([
      'General', 'Connections', 'Data and backup', 'Sessions and run queue', 'Review', 'Time and usage', 'Notifications', 'Appearance', 'Keyboard', 'Help', 'About'
    ])
  })

  it('opens Help on the topic the address names, with the topics listed under Help in the rail', async () => {
    const page = await mountAt('/settings/help/steps')

    expect(page.get('[data-testid="help-panel"]').attributes('aria-label')).toBe('Steps')
    expect(page.get('[data-testid="settings-help-topic-steps"]').attributes('aria-current')).toBe('page')
    expect(page.findAll('[data-testid="settings-help-topics"] a')).toHaveLength(9)
  })

  it('lists the Help topics only while Help is open', async () => {
    const page = await mountAt('/settings/appearance')

    expect(page.find('[data-testid="settings-help-topics"]').exists()).toBe(false)
  })

  it('opens the section the address names', async () => {
    const page = await mountAt('/settings/about')

    expect(page.find('[data-testid="version-section"]').exists()).toBe(true)
    expect(page.get('[data-testid="settings-rail-about"]').attributes('aria-current')).toBe('page')
  })

  it('opens Data and backup with the unused images above the backups and Langfuse', async () => {
    const page = await mountAt('/settings/data')

    expect(page.findAll('main [data-testid]').map((part) => part.attributes('data-testid'))).toEqual([
      'picture-cleanup', 'form-panel', 'backups-section', 'form-panel', 'langfuse-section'
    ])
    expect(page.findAll('main h2').map((title) => title.text())).toEqual(['Backups', 'Langfuse'])
  })

  it('opens Sessions with what both providers share above a column for each', async () => {
    const page = await mountAt('/settings/sessions')

    expect(page.find('main [data-testid="shared-session-settings"]').exists()).toBe(true)
    expect(page.findAll('[data-testid="provider-columns"] > div').map((column) => column.attributes('data-testid'))).toEqual([
      'claude-section', 'codex-settings'
    ])
  })

  it('saves the session provider for new sessions and keeps both columns open', async () => {
    const page = await mountAt('/settings/sessions')
    const choice = page.get('[data-testid="session-provider"]')
    expect(choice.get('[data-testid="segment-CLAUDE"]').attributes('aria-pressed')).toBe('true')

    await choice.get('[data-testid="segment-CODEX"]').trigger('click')

    expect(preferredProvider()).toBe('CODEX')
    expect(page.find('[data-testid="claude-section"]').exists()).toBe(true)
    expect(page.find('[data-testid="codex-settings"]').exists()).toBe(true)
  })

  it('changes behaviour from Appearance: the density Work reads and the motion on the document', async () => {
    const page = await mountAt('/settings/appearance')

    await page.get('[data-testid="segment-compact"]').trigger('click')
    await page.get('[data-testid="segment-reduced"]').trigger('click')
    await flushPromises()

    expect(useProjectTasksStore().density).toBe('compact')
    expect(usePreferencesStore().motion).toBe('reduced')
  })

  it('sets the review choices and the time goals in the preferences store', async () => {
    const preferences = usePreferencesStore()
    let page = await mountAt('/settings/review')
    const boxes = page.findAll('input[type="checkbox"]')
    await boxes[0]!.setValue(false)
    await boxes[1]!.setValue(true)
    expect(preferences.reviewAutoAdvance).toBe(false)
    expect(preferences.reviewConfirmSendBack).toBe(true)

    page = await mountAt('/settings/time')
    await page.get('#workday-goal').setValue('6')
    await page.get('#usage-warning').setValue('65')
    expect(preferences.workdayGoalHours).toBe(6)
    expect(preferences.usageWarningPercent).toBe(65)
  })
})
