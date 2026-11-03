import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import NavigatorTaskRow from '@/components/console/NavigatorTaskRow.vue'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'
import { chatSession } from './features/chat/support'
import { taskFixture } from './support/taskFixture'

/** The session mark under a task's mark in the navigator. */
const task = taskFixture('t-1' as TaskId)

function mountRow(openTask: Task = task) {
  return mount(NavigatorTaskRow, {
    props: { task: openTask, selected: false, running: false, showContext: false, showCompany: false }
  })
}

beforeEach(() => setActivePinia(createPinia()))

describe('NavigatorTaskRow session mark', () => {
  it('draws nothing for a task no session was opened on', () => {
    expect(mountRow().find('[data-testid=task-session-mark]').exists()).toBe(false)
  })

  it('shows a playing mark while a session is open on the task', () => {
    useTerminalStore().upsert(chatSession())

    expect(mountRow().get('[data-testid=task-session-mark]').attributes('data-state')).toBe('LIVE')
  })

  it('shows a resting mark once the session has ended', () => {
    useTerminalStore().upsert(chatSession({ live: false }))

    expect(mountRow().get('[data-testid=task-session-mark]').attributes('data-state')).toBe('ENDED')
  })

  it('asks for the approval a session waits for, and says so to a screen reader', () => {
    useTerminalStore().upsert(chatSession({ awaitingPermission: true }))
    const row = mountRow()

    expect(row.get('[data-testid=task-session-mark]').attributes('data-state')).toBe('APPROVAL')
    expect(row.text()).toContain('A session is waiting for your approval')
  })

  it('shows a check once the work a session ran on is handed in', () => {
    useTerminalStore().upsert(chatSession())
    const claimedTask: Task = { ...task, status: 'IN_PROGRESS', reviewActive: true, reviewState: 'CLAIMED' }

    expect(mountRow(claimedTask).get('[data-testid=task-session-mark]').attributes('data-state')).toBe('CLAIMED')
  })
})
