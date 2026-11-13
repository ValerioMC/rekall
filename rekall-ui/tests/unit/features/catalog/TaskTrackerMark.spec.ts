import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import TaskTrackerMark from '@/features/catalog/components/TaskTrackerMark.vue'
import { useConsoleStore } from '@/stores/console.store'
import { taskFixture } from '../../support/taskFixture'
import type { TaskId } from '@/model/branded'

const syncTaskTracker = vi.fn()
const fetchTask = vi.fn()

vi.mock('@/features/catalog/api/catalog.api', async (importOriginal) => ({
  ...(await importOriginal<object>()),
  syncTaskTracker: (...args: unknown[]) => syncTaskTracker(...args),
  fetchTask: (...args: unknown[]) => fetchTask(...args)
}))

const TASK = 't1' as TaskId

type Tracked = {
  trackerIssueKey: string | null
  trackerIssueUrl: string | null
  trackerSyncError: string | null
  trackerContentSynced: boolean
}

const held: Tracked = {
  trackerIssueKey: 'ENG-12',
  trackerIssueUrl: 'https://example.atlassian.net/browse/ENG-12',
  trackerSyncError: null,
  trackerContentSynced: true
}

const render = (task: Tracked) => mount(TaskTrackerMark, { props: { task: { id: TASK, ...task } } })

describe('the task tracker mark', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    syncTaskTracker.mockReset()
    fetchTask.mockReset()
  })

  afterEach(() => {
    delete window.rekallDesktop
  })

  it('shows nothing for a task the task manager does not hold', () => {
    const none = { trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true }

    expect(render(none).html()).toBe('<!--v-if-->')
  })

  it('links the issue the task manager holds', () => {
    const link = render(held).get('[data-testid="tracker-issue-link"]')

    expect(link.attributes('href')).toBe('https://example.atlassian.net/browse/ENG-12')
    expect(link.text()).toContain('ENG-12')
  })

  it('says why the last write failed', () => {
    const mark = render({ trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: 'GitLab rejected the stored credential (HTTP 401).', trackerContentSynced: false })

    expect(mark.find('[data-testid="tracker-issue-link"]').exists()).toBe(false)
    expect(mark.get('[data-testid="tracker-sync-error"]').attributes('title')).toContain('HTTP 401')
    expect(mark.get('[data-testid="tracker-sync"]').attributes('data-state')).toBe('failed')
  })

  it('hands the link to the desktop window, which opens it in the browser', async () => {
    const openExternalLink = vi.fn().mockResolvedValue(undefined)
    window.rekallDesktop = { pickFolder: vi.fn(), openExternalLink }

    await render({ ...held, trackerIssueUrl: 'https://github.com/acme/app/issues/12' }).get('[data-testid="tracker-issue-link"]').trigger('click')

    expect(openExternalLink).toHaveBeenCalledWith('https://github.com/acme/app/issues/12')
  })

  it('shows a check, and writes nothing when pressed, while the issue holds what the task holds', async () => {
    const mark = render(held)
    const key = mark.get('[data-testid="tracker-sync"]')

    await key.trigger('click')

    expect(key.attributes('data-state')).toBe('synced')
    expect(key.attributes('title')).toBe('Synced with issue ENG-12')
    expect(mark.find('[data-testid="tracker-sync-changed"]').exists()).toBe(false)
    expect(syncTaskTracker).not.toHaveBeenCalled()
  })

  it('marks the key with a dot once the task changed since the last sync', () => {
    const mark = render({ ...held, trackerContentSynced: false })

    expect(mark.get('[data-testid="tracker-sync"]').attributes('data-state')).toBe('pending')
    expect(mark.get('[data-testid="tracker-sync"]').attributes('title')).toContain('Changed since the last sync')
    expect(mark.find('[data-testid="tracker-sync-changed"]').exists()).toBe(true)
  })

  it('writes the task when pressed and spins until the server answers', async () => {
    let answer: (value: unknown) => void = () => undefined
    syncTaskTracker.mockReturnValue(new Promise((resolve) => (answer = resolve)))
    const mark = render({ ...held, trackerContentSynced: false })
    const key = mark.get('[data-testid="tracker-sync"]')

    await key.trigger('click')
    await key.trigger('click')

    expect(syncTaskTracker).toHaveBeenCalledTimes(1)
    expect(syncTaskTracker).toHaveBeenCalledWith(TASK)
    expect(key.attributes('data-state')).toBe('syncing')
    expect(key.attributes('aria-busy')).toBe('true')

    answer({ ...taskFixture(TASK), trackerContentSynced: true })
    await flushPromises()

    expect(key.attributes('aria-busy')).toBe('false')
  })

  it('offers the key to create the issue when the task has none', () => {
    const mark = render({ trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: 'GitHub could not take the issue: boom.', trackerContentSynced: false })

    expect(mark.get('[data-testid="tracker-sync"]').attributes('aria-label')).toBe('Create the issue in the task manager')
  })

  it('reads the task again and keeps the key usable when the write is refused', async () => {
    syncTaskTracker.mockRejectedValue(new Error('GitHub rejected the stored credential (HTTP 401).'))
    const refused = { ...taskFixture(TASK), trackerSyncError: 'GitHub rejected the stored credential (HTTP 401).' }
    fetchTask.mockResolvedValue(refused)
    const store = useConsoleStore()
    store.tasks = [taskFixture(TASK)]
    const mark = render({ ...held, trackerContentSynced: false })

    await mark.get('[data-testid="tracker-sync"]').trigger('click')
    await flushPromises()

    expect(fetchTask).toHaveBeenCalledWith(TASK)
    expect(mark.get('[data-testid="tracker-sync"]').attributes('aria-busy')).toBe('false')
    expect(store.tasks[0]?.trackerSyncError).toContain('HTTP 401')
  })
})
