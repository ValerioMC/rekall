import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import LogCommitButton from '@/components/console/LogCommitButton.vue'
import { fetchRecentCommits, recordCommit, recordLatestCommit } from '@/api/commitReference.api'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { CommitReference, RecentCommit } from '@/model/commitReference'
import type { TaskId, TaskStepId } from '@/model/branded'

vi.mock('@/api/commitReference.api', () => ({
  fetchRecentCommits: vi.fn(),
  recordCommit: vi.fn(),
  recordLatestCommit: vi.fn()
}))

const taskId = 't1' as TaskId
const stepId = 's1' as TaskStepId

const tip = 'a'.repeat(40)
const earlier = 'b'.repeat(40)
const oldest = 'c'.repeat(40)

function recent(): RecentCommit[] {
  return [
    { hash: tip, subject: 'Polish the picker', committedAt: '2026-09-15T09:00:00Z' },
    { hash: earlier, subject: 'Add the picker', committedAt: '2026-09-14T09:00:00Z' },
    { hash: oldest, subject: 'Add the readme', committedAt: '2026-09-13T09:00:00Z' }
  ]
}

function logged(hash: string, comment: string, step: TaskStepId | null = null): CommitReference {
  return {
    id: `ref-${hash.slice(0, 4)}`,
    taskId,
    stepId: step,
    stepTitle: step ? 'The step' : null,
    commitHash: hash,
    comment,
    inContext: false,
    createdAt: '2026-09-15T10:00:00Z'
  }
}

function render(props: Partial<InstanceType<typeof LogCommitButton>['$props']> = {}): VueWrapper {
  return mount(LogCommitButton, {
    props: { taskId, folder: '/repo/vega', ...props },
    attachTo: document.body
  })
}

/** The picker teleports to <body>, so it is looked up on the document, not on the wrapper. */
function picker(): HTMLElement | null {
  return document.querySelector('[data-testid="commit-picker"]')
}

function rows(): HTMLButtonElement[] {
  return Array.from(document.querySelectorAll<HTMLButtonElement>('[data-testid="commit-picker-row"]'))
}

describe('LogCommitButton', () => {
  let wrapper: VueWrapper | null = null

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(fetchRecentCommits).mockReset().mockResolvedValue(recent())
    vi.mocked(recordCommit).mockReset()
    vi.mocked(recordLatestCommit).mockReset()
  })

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
    document.body.innerHTML = ''
  })

  it('logs the tip with the main button, as before', async () => {
    vi.mocked(recordLatestCommit).mockResolvedValue(logged(tip, 'Polish the picker'))
    wrapper = render()

    await wrapper.get('[data-testid="log-commit"]').trigger('click')
    await flushPromises()

    expect(recordLatestCommit).toHaveBeenCalledWith(taskId, null)
    expect(recordCommit).not.toHaveBeenCalled()
    expect(useConsoleStore().commitReferences.map((reference) => reference.commitHash)).toEqual([tip])
    expect(picker()).toBeNull()
  })

  it('opens the recent log from the chevron, newest first, with the tip marked', async () => {
    wrapper = render()

    await wrapper.get('[data-testid="pick-commit"]').trigger('click')
    await flushPromises()

    expect(fetchRecentCommits).toHaveBeenCalledWith(taskId)
    expect(picker()).not.toBeNull()
    expect(rows().map((row) => row.dataset.hash)).toEqual([tip, earlier, oldest])
    expect(
      Array.from(document.querySelectorAll('[data-testid="commit-picker-subject"]')).map((subject) =>
        subject.textContent?.trim()
      )
    ).toEqual(['Polish the picker', 'Add the picker', 'Add the readme'])
    expect(document.querySelector('[data-testid="commit-picker-hash"]')?.textContent?.trim()).toBe('aaaaaaa')
    expect(picker()?.textContent).toContain('tip')
  })

  it('logs the commit a row was picked for, against the step it sits in, and closes', async () => {
    vi.mocked(recordCommit).mockResolvedValue(logged(earlier, 'Add the picker', stepId))
    wrapper = render({ stepId })

    await wrapper.get('[data-testid="pick-commit"]').trigger('click')
    await flushPromises()
    rows()[1]!.click()
    await flushPromises()

    expect(recordCommit).toHaveBeenCalledWith(taskId, stepId, earlier)
    expect(useConsoleStore().commitReferences[0]?.commitHash).toBe(earlier)
    expect(picker()).toBeNull()
    expect(wrapper.get('[data-testid="log-commit"]').text()).toContain('bbbbbbb')
  })

  it('logs a pasted hash, and refuses anything that is not hex before it is sent', async () => {
    vi.mocked(recordCommit).mockResolvedValue(logged('deadbeef', 'From far back'))
    wrapper = render()
    await wrapper.get('[data-testid="pick-commit"]').trigger('click')
    await flushPromises()

    const field = document.querySelector<HTMLInputElement>('[data-testid="commit-picker-paste"]')!
    const submit = document.querySelector<HTMLButtonElement>('[data-testid="commit-picker-log-pasted"]')!

    field.value = 'not a hash'
    field.dispatchEvent(new Event('input'))
    await flushPromises()
    expect(submit.disabled).toBe(true)

    field.value = '  deadbeef '
    field.dispatchEvent(new Event('input'))
    await flushPromises()
    expect(submit.disabled).toBe(false)
    submit.form!.dispatchEvent(new Event('submit', { cancelable: true }))
    await flushPromises()

    expect(recordCommit).toHaveBeenCalledWith(taskId, null, 'deadbeef')
    expect(picker()).toBeNull()
  })

  it('marks the rows already logged against this task/step and will not pick them again', async () => {
    useConsoleStore().applyCommitReference(logged(earlier, 'Add the picker'))
    useConsoleStore().applyCommitReference(logged(oldest, 'Add the readme', stepId))
    wrapper = render()

    await wrapper.get('[data-testid="pick-commit"]').trigger('click')
    await flushPromises()

    const [first, second, third] = rows()
    expect(first!.disabled).toBe(false)
    expect(second!.disabled).toBe(true)
    expect(third!.disabled).toBe(false)
    expect(document.querySelectorAll('[data-testid="commit-picker-logged"]')).toHaveLength(1)
  })

  it('shows the reason when the log cannot be read, and stays open', async () => {
    vi.mocked(fetchRecentCommits).mockRejectedValue(new Error('Could not read the log in /repo/vega'))
    wrapper = render()

    await wrapper.get('[data-testid="pick-commit"]').trigger('click')
    await flushPromises()

    expect(document.querySelector('[data-testid="commit-picker-failure"]')?.textContent).toContain(
      'Could not read the log'
    )
    expect(rows()).toHaveLength(0)
  })

  it('keeps the picker open and reports the failure when the picked commit is refused', async () => {
    vi.mocked(recordCommit).mockRejectedValue(new Error("No commit 'bbbb' in /repo/vega"))
    wrapper = render()
    await wrapper.get('[data-testid="pick-commit"]').trigger('click')
    await flushPromises()

    rows()[1]!.click()
    await flushPromises()

    expect(picker()).not.toBeNull()
    expect(useToastStore().toasts.some((toast) => toast.message.includes('No commit'))).toBe(true)
  })

  it('closes on Escape and on a click outside, and toggles closed from the chevron', async () => {
    wrapper = render()
    const chevron = wrapper.get('[data-testid="pick-commit"]')

    await chevron.trigger('click')
    await flushPromises()
    expect(picker()).not.toBeNull()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()
    expect(picker()).toBeNull()

    await chevron.trigger('click')
    await flushPromises()
    document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    await flushPromises()
    expect(picker()).toBeNull()

    await chevron.trigger('click')
    await flushPromises()
    expect(picker()).not.toBeNull()
    await chevron.trigger('click')
    await flushPromises()
    expect(picker()).toBeNull()
  })

  it('refuses to open the picker without a project folder, same as logging', async () => {
    wrapper = render({ folder: null })

    await wrapper.get('[data-testid="pick-commit"]').trigger('click')
    await flushPromises()

    expect(picker()).toBeNull()
    expect(fetchRecentCommits).not.toHaveBeenCalled()
    expect(useToastStore().toasts.some((toast) => toast.message.includes('folder'))).toBe(true)
  })
})
