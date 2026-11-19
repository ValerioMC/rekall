import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import DatabaseSection from '@/features/settings/components/DatabaseSection.vue'

let wrapper: ReturnType<typeof mount> | undefined

function mountPanel(): ReturnType<typeof mount> {
  wrapper = mount(DatabaseSection, { attachTo: document.body })
  return wrapper
}

/**
 * The Settings surface for the database registry: what is listed, what only the active entry
 * gets (no Forget button), and that forgetting always goes through the same confirmation every
 * other destructive action in this application uses.
 */
const fetchDatabaseStatus = vi.fn()
const forgetDatabase = vi.fn()
const renameDatabase = vi.fn()
const activateDatabase = vi.fn()

vi.mock('@/features/settings/api/settings.api', () => ({
  fetchDatabaseStatus: () => fetchDatabaseStatus(),
  forgetDatabase: (...args: unknown[]) => forgetDatabase(...args),
  renameDatabase: (...args: unknown[]) => renameDatabase(...args),
  activateDatabase: (...args: unknown[]) => activateDatabase(...args),
  addDatabase: vi.fn(),
  checkFolder: vi.fn()
}))

const local = { id: '1', label: 'Local', path: '/a', active: true, reachable: true, addedAt: 't', lastUsedAt: 't' }
const backup = { id: '2', label: 'Backup', path: '/b', active: false, reachable: true, addedAt: 't', lastUsedAt: 't' }

function findButton(wrapper: ReturnType<typeof mount>, text: string) {
  const button = wrapper.findAll('button').find((candidate) => candidate.text() === text)
  if (!button) throw new Error(`No button with text "${text}"`)
  return button
}

describe('DatabaseSection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    fetchDatabaseStatus.mockReset()
    forgetDatabase.mockReset()
    renameDatabase.mockReset()
    activateDatabase.mockReset()
    fetchDatabaseStatus.mockResolvedValue({ status: 'READY', active: local, databases: [local, backup] })
  })

  afterEach(() => {
    wrapper?.unmount()
    wrapper = undefined
  })

  it('lists every registered database and marks the active one', async () => {
    const wrapper = mountPanel()
    await flushPromises()

    const rows = wrapper.findAll('[data-testid="database-row"]')
    expect(rows).toHaveLength(2)
    expect(rows[0]!.text()).toContain('In use')
    expect(rows[1]!.text()).not.toContain('In use')
  })

  it('never offers to forget the database currently in use', async () => {
    const wrapper = mountPanel()
    await flushPromises()

    const [activeRow, otherRow] = wrapper.findAll('[data-testid="database-row"]')
    expect(activeRow!.text()).not.toContain('Forget')
    expect(otherRow!.text()).toContain('Forget')
  })

  it('renames a database', async () => {
    renameDatabase.mockResolvedValue({ ...local, label: 'Renamed' })
    const wrapper = mountPanel()
    await flushPromises()

    await findButton(wrapper, 'rename').trigger('click')
    await wrapper.get('input[aria-label="Rename database"]').setValue('Renamed')
    await findButton(wrapper, 'Save').trigger('click')
    await flushPromises()

    expect(renameDatabase).toHaveBeenCalledWith('1', 'Renamed')
  })

  it('pressing Enter in the rename field saves it, ', async () => {
    renameDatabase.mockResolvedValue({ ...local, label: 'Renamed' })
    const wrapper = mountPanel()
    await flushPromises()

    await findButton(wrapper, 'rename').trigger('click')
    const input = wrapper.get('input[aria-label="Rename database"]')
    await input.setValue('Renamed')
    await input.trigger('keydown', { key: 'Enter' })
    await flushPromises()

    expect(renameDatabase).toHaveBeenCalledWith('1', 'Renamed')
  })

  it('pressing Escape in the rename field cancels the rename', async () => {
    const wrapper = mountPanel()
    await flushPromises()

    await findButton(wrapper, 'rename').trigger('click')
    await wrapper.get('input[aria-label="Rename database"]').trigger('keydown', { key: 'Escape' })

    expect(wrapper.find('input[aria-label="Rename database"]').exists()).toBe(false)
  })

  it('forgets a database only after the confirmation is accepted', async () => {
    forgetDatabase.mockResolvedValue(undefined)
    const wrapper = mountPanel()
    await flushPromises()

    await findButton(wrapper, 'Forget').trigger('click')
    expect(forgetDatabase).not.toHaveBeenCalled()

    const dialog = wrapper.get('[role="alertdialog"]')
    expect(dialog.text()).toContain('Backup')
    await dialog.findAll('button').find((button) => button.text() === 'Forget')!.trigger('click')
    await flushPromises()

    expect(forgetDatabase).toHaveBeenCalledWith('2')
  })

  it('cancelling the confirmation leaves the database registered', async () => {
    const wrapper = mountPanel()
    await flushPromises()

    await findButton(wrapper, 'Forget').trigger('click')
    await findButton(wrapper, 'Keep it').trigger('click')

    expect(forgetDatabase).not.toHaveBeenCalled()
    expect(wrapper.find('[role="alertdialog"]').exists()).toBe(false)
  })
})
