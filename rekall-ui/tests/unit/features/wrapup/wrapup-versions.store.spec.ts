import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/wrapup/api/wrapup-versions.api'
import { useWrapupVersionsStore } from '@/features/wrapup/store/wrapup-versions.store'
import { useConsoleStore } from '@/stores/console.store'
import { TASK, detail, version } from './fixtures'

vi.mock('@/features/wrapup/api/wrapup-versions.api')

describe('the wrapup versions store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('keeps the versions of a task in the order the server gave them', async () => {
    vi.mocked(api.fetchWrapupVersions).mockResolvedValue([version(2), version(1)])
    const store = useWrapupVersionsStore()

    await store.load(TASK)

    expect(store.versionsOf(TASK).map((v) => v.number)).toEqual([2, 1])
  })

  it('has no versions for a task it has not read', () => {
    expect(useWrapupVersionsStore().versionsOf(TASK)).toEqual([])
  })

  it('restores a version by writing its whole text as the wrapup and reading the versions again', async () => {
    vi.mocked(api.fetchWrapupVersion).mockResolvedValue(detail(1))
    vi.mocked(api.fetchWrapupVersions).mockResolvedValue([version(3, 'HAND'), version(2), version(1)])
    const saveWrapupBody = vi.spyOn(useConsoleStore(), 'saveWrapupBody').mockResolvedValue()
    const store = useWrapupVersionsStore()

    await store.restore(TASK, version(1).id)

    expect(saveWrapupBody).toHaveBeenCalledWith(TASK, 'Whole text of state 1')
    expect(store.versionsOf(TASK).map((v) => v.number)).toEqual([3, 2, 1])
  })
})
