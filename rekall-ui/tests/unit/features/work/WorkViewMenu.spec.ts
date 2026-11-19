import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import WorkViewMenu from '@/features/work/components/WorkViewMenu.vue'
import { useWorkStore } from '@/features/work/store/work.store'
import { installStorage } from '../../support/memoryStorage'

let wrapper: VueWrapper | null = null

function render(showsDensity: boolean, showsQuickTasks: boolean): VueWrapper {
  wrapper = mount(WorkViewMenu, { props: { showsDensity, showsQuickTasks }, global: { plugins: [createPinia()] }, attachTo: document.body })
  return wrapper
}

beforeEach(() => {
  installStorage()
  setActivePinia(createPinia())
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  delete (window as { localStorage?: unknown }).localStorage
})

describe('the Work view menu', () => {
  it('stays closed until its key is pressed and closes on Escape', async () => {
    const menu = render(true, false)
    expect(menu.find('[data-testid="work-view-panel"]').exists()).toBe(false)

    await menu.get('[data-testid="work-view-trigger"]').trigger('click')
    expect(menu.find('[data-testid="work-view-panel"]').exists()).toBe(true)

    await menu.get('[data-testid="work-view-menu"]').trigger('keydown', { key: 'Escape' })
    expect(menu.find('[data-testid="work-view-panel"]').exists()).toBe(false)
  })

  it('sets the row density in the Work store', async () => {
    const menu = render(true, false)
    await menu.get('[data-testid="work-view-trigger"]').trigger('click')

    await menu.get('[data-testid="segment-compact"]').trigger('click')

    expect(useWorkStore().density).toBe('compact')
  })

  it('offers only what applies: no density on the board, the quick-task filter only when there are quick tasks', async () => {
    const board = render(false, true)
    await board.get('[data-testid="work-view-trigger"]').trigger('click')
    expect(board.find('[data-testid="segment-compact"]').exists()).toBe(false)
    expect(board.find('[data-testid="hide-quick-tasks"]').exists()).toBe(true)
    board.unmount()

    const ledger = render(true, false)
    await ledger.get('[data-testid="work-view-trigger"]').trigger('click')
    expect(ledger.find('[data-testid="segment-compact"]').exists()).toBe(true)
    expect(ledger.find('[data-testid="hide-quick-tasks"]').exists()).toBe(false)
  })
})
