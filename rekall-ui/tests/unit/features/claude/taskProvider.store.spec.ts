import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { setPreferredProvider } from '@/common/config/session-launch'
import { useTaskProviderStore } from '@/features/claude/store/taskProvider.store'
import { installStorage } from '../../support/memoryStorage'
import type { TaskId } from '@/model/branded'

const SWITCHED = 't-1' as TaskId
const UNTOUCHED = 't-2' as TaskId

describe('the provider a task is switched to', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
  })

  it('follows the Settings default on a task never switched', () => {
    setPreferredProvider('CODEX')

    expect(useTaskProviderStore().providerFor(UNTOUCHED)).toBe('CODEX')
  })

  it('belongs to that task alone', () => {
    const store = useTaskProviderStore()

    store.choose(SWITCHED, 'CODEX')

    expect(store.providerFor(SWITCHED)).toBe('CODEX')
    expect(store.providerFor(UNTOUCHED)).toBe('CLAUDE')
  })

  it('is still there after the console reloads, whatever the default becomes', () => {
    useTaskProviderStore().choose(SWITCHED, 'CLAUDE')
    setActivePinia(createPinia())
    setPreferredProvider('CODEX')

    expect(useTaskProviderStore().providerFor(SWITCHED)).toBe('CLAUDE')
  })

  it('holds for the visit when the browser refuses to store it', () => {
    installStorage({
      setItem: () => {
        throw new Error('storage is full')
      }
    })
    const store = useTaskProviderStore()

    store.choose(SWITCHED, 'CODEX')

    expect(store.providerFor(SWITCHED)).toBe('CODEX')
  })
})
