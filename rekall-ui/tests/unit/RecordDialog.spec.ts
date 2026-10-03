import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import RecordDialog from '@/components/console/RecordDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import { taskDraft } from '@/model/record-draft'
import type { Task } from '@/model/catalog'
import type { ProjectId } from '@/model/branded'

/**
 * The save button while a save is in flight: disabled and inert, with no spinner, because the
 * dialog usually closes before any animation could finish. The write path is stubbed.
 */
const projectId = 'p1' as ProjectId

let pinia: Pinia
let wrapper: VueWrapper | null = null

function pendingCreate(): { resolve: () => void; create: ReturnType<typeof vi.fn> } {
  let resolve: () => void = () => undefined
  const pending = new Promise<Task>((settle) => {
    resolve = () => settle({} as Task)
  })
  return { resolve, create: vi.fn().mockReturnValue(pending) }
}

function render(): VueWrapper {
  wrapper = mount(RecordDialog, {
    attachTo: document.body,
    props: { draft: { ...taskDraft(projectId), title: 'Report builder', label: 'report-builder' } },
    global: { plugins: [pinia] }
  })
  return wrapper
}

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('RecordDialog save button', () => {
  it('disables itself without a spinner while the save runs', async () => {
    const store = useConsoleStore()
    const { resolve, create } = pendingCreate()
    store.createTask = create
    const dialog = render()
    const save = dialog.find('[data-testid="record-save"]')

    await save.trigger('click')

    expect(save.attributes('disabled')).toBeDefined()
    expect(save.find('.animate-spin').exists()).toBe(false)
    resolve()
    await flushPromises()
    expect(dialog.emitted('close')).toHaveLength(1)
  })

  it('ignores a second submit while the first is still saving', async () => {
    const store = useConsoleStore()
    const { resolve, create } = pendingCreate()
    store.createTask = create
    const dialog = render()

    await dialog.find('[data-testid="record-save"]').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', metaKey: true }))
    resolve()
    await flushPromises()

    expect(create).toHaveBeenCalledTimes(1)
  })
})
