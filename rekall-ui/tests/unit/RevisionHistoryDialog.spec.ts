import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import RevisionHistoryDialog from '@/components/console/RevisionHistoryDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskRevision } from '@/model/revision'
import type { TaskId } from '@/model/branded'

const fetchRevisions = vi.fn<() => Promise<TaskRevision[]>>()
vi.mock('@/api/revisions.api', () => ({
  fetchRevisions: () => fetchRevisions(),
  restoreRevision: vi.fn()
}))

/** The editor is CodeMirror underneath; the dialog only has to hand it the body it previews. */
vi.mock('@/components/ui/AppMarkdownEditor.vue', () => ({
  default: { props: ['modelValue', 'readonly'], template: '<pre data-testid="preview">{{ modelValue }}</pre>' }
}))

const taskId = 't1' as TaskId

const revision = (id: string, body: string, replacedAt: string): TaskRevision => ({
  id,
  taskId,
  kind: 'WRAPUP',
  bodyMarkdown: body,
  writtenBy: 'HAND',
  writtenAt: '2026-09-20T08:00:00Z',
  replacedAt
})

function render() {
  return mount(RevisionHistoryDialog, {
    props: { taskId, taskTitle: 'Report builder', kind: 'WRAPUP' },
    attachTo: document.body
  })
}

describe('RevisionHistoryDialog', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    fetchRevisions.mockReset()
  })

  it('lists the kept versions newest first and previews the newest', async () => {
    fetchRevisions.mockResolvedValue([
      revision('r2', 'The newer one.', '2026-09-21T10:00:00Z'),
      revision('r1', 'The older one.', '2026-09-20T10:00:00Z')
    ])

    const wrapper = render()
    await flushPromises()

    expect(wrapper.findAll('[data-testid="revision-row"]')).toHaveLength(2)
    expect(wrapper.find('[data-testid="preview"]').text()).toBe('The newer one.')

    await wrapper.findAll('[data-testid="revision-row"]')[1]!.trigger('click')
    expect(wrapper.find('[data-testid="preview"]').text()).toBe('The older one.')
    wrapper.unmount()
  })

  it('restores the chosen version through the store and says so to its owner', async () => {
    fetchRevisions.mockResolvedValue([revision('r1', 'The older one.', '2026-09-20T10:00:00Z')])
    const store = useConsoleStore()
    store.restoreRevision = vi.fn().mockResolvedValue('WRAPUP')

    const wrapper = render()
    await flushPromises()
    await wrapper.find('[data-testid="revision-restore"]').trigger('click')
    await flushPromises()

    expect(store.restoreRevision).toHaveBeenCalledWith(taskId, 'r1')
    expect(wrapper.emitted('restored')).toHaveLength(1)
    wrapper.unmount()
  })

  it('says there is nothing yet when nothing has been replaced', async () => {
    fetchRevisions.mockResolvedValue([])

    const wrapper = render()
    await flushPromises()

    expect(wrapper.find('[data-testid="revision-history-empty"]').exists()).toBe(true)
    wrapper.unmount()
  })
})
