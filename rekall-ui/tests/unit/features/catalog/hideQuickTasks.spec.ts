import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { taskFixture } from '../../support/taskFixture'
import type { TaskId } from '@/model/branded'

describe('the navigator list and quick tasks', () => {
  beforeEach(() => setActivePinia(createPinia()))

  function storeWithOneOfEach() {
    const store = useConsoleStore()
    store.tasks = [{ ...taskFixture('a' as TaskId), quick: true }, taskFixture('b' as TaskId)]
    return store
  }

  it('lists quick tasks by default', () => {
    expect(storeWithOneOfEach().visibleTasks.map((task) => task.id)).toEqual(['a', 'b'])
  })

  it('leaves them out once the reader hides them', () => {
    const store = storeWithOneOfEach()
    store.hideQuickTasks = true
    expect(store.visibleTasks.map((task) => task.id)).toEqual(['b'])
  })
})
