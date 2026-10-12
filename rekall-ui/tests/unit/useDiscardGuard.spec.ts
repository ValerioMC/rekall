import { describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import { useDiscardGuard } from '@/composables/useDiscardGuard'

describe('the discard guard', () => {
  it('runs an action at once when nothing is unsaved', () => {
    const action = vi.fn()
    const { guard, asking } = useDiscardGuard(ref(false))

    guard(action)

    expect(action).toHaveBeenCalledOnce()
    expect(asking.value).toBe(false)
  })

  it('holds the action until the person confirms', () => {
    const action = vi.fn()
    const { guard, asking, confirm } = useDiscardGuard(ref(true))

    guard(action)
    expect(action).not.toHaveBeenCalled()
    expect(asking.value).toBe(true)

    confirm()
    expect(action).toHaveBeenCalledOnce()
    expect(asking.value).toBe(false)
  })

  it('drops the action when the person keeps their edits', () => {
    const action = vi.fn()
    const { guard, cancel, confirm } = useDiscardGuard(ref(true))

    guard(action)
    cancel()
    confirm()

    expect(action).not.toHaveBeenCalled()
  })

  it('lets a route change through at once when clean, and waits for the answer when not', async () => {
    const clean = useDiscardGuard(ref(false))
    expect(await clean.askToLeave()).toBe(true)

    const dirty = useDiscardGuard(ref(true))
    const answer = dirty.askToLeave()
    dirty.cancel()
    expect(await answer).toBe(false)

    const again = dirty.askToLeave()
    dirty.confirm()
    expect(await again).toBe(true)
  })
})
