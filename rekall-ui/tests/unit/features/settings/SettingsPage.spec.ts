import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { installStorage } from '../../support/memoryStorage'
import { flushPromises } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import SettingsPage from '@/features/settings/components/SettingsPage.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useWorkStore } from '@/features/work/store/work.store'
import { mountWithHeaderChrome } from '../../support/mountWithHeaderChrome'

vi.mock('@/features/settings/components/DatabaseSection.vue', () => ({ default: { template: '<div data-testid="database-section" />' } }))
vi.mock('@/features/settings/components/VersionSection.vue', () => ({ default: { template: '<div data-testid="version-section" />' } }))

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

  it('changes behaviour from Appearance: the density Work reads and the motion on the document', async () => {
    const page = await mountAt('/settings/appearance')

    await page.get('[data-testid="segment-compact"]').trigger('click')
    await page.get('[data-testid="segment-reduced"]').trigger('click')
    await flushPromises()

    expect(useWorkStore().density).toBe('compact')
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
