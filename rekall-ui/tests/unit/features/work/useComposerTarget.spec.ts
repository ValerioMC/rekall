import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useComposerTarget } from '@/features/work/composables/useComposerTarget'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import type { ProjectId } from '@/model/branded'
import { project, task } from './work-fixtures'

const orion = 'p2' as ProjectId

function seed() {
  setActivePinia(createPinia())
  const store = useConsoleStore()
  store.projects = [project('p1', 'Vega'), project('p2', 'Orion')]
  store.tasks = [
    task('PR-1', { updatedAt: '2026-09-01T10:00:00Z' }),
    task('PR-2', { projectId: orion, projectTitle: 'Orion', updatedAt: '2026-09-20T10:00:00Z' })
  ]
  return useComposerTarget()
}

describe('the project a new task goes to', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
  })

  it('is the project worked on most recently when no project is in view', () => {
    const { target, choices } = seed()

    expect(target.value?.id).toBe(orion)
    expect(choices.value.map((choice) => choice.id)).toEqual(['p1', 'p2'])
  })

  it('is the one picked, until another is', () => {
    const { target, choose } = seed()

    choose('p1' as ProjectId)

    expect(target.value?.id).toBe('p1')
  })

  it('is the project in view, with nothing to pick from', () => {
    const { target, choices } = seed()

    useWorkStore().focusProject('p1' as ProjectId)

    expect(target.value?.id).toBe('p1')
    expect(choices.value).toEqual([])
  })
})
