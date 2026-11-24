import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useClaudeCommandUpdateStore } from '@/features/claude/store/claudeCommandUpdate.store'
import type { CommandUpdateState } from '@/features/claude/model/claude'

const fetchCommandUpdateState = vi.fn()
const fetchClaudeInstallation = vi.fn()

vi.mock('@/features/claude/api/claude.api', () => ({
  fetchCommandUpdateState: () => fetchCommandUpdateState(),
  fetchClaudeInstallation: () => fetchClaudeInstallation(),
  installClaudeIntegration: vi.fn()
}))

const POLL_INTERVAL_MILLIS = 1000

function reading(state: CommandUpdateState['state']): CommandUpdateState {
  return { state }
}

describe('claude command update store', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    fetchCommandUpdateState.mockReset()
    fetchClaudeInstallation.mockReset()
    fetchClaudeInstallation.mockResolvedValue({})
  })

  afterEach(() => vi.useRealTimers())

  it('is not updating before the state is read', () => {
    expect(useClaudeCommandUpdateStore().updating).toBe(false)
  })

  it('loads the state the server reports', async () => {
    fetchCommandUpdateState.mockResolvedValue(reading('UPDATING'))
    const update = useClaudeCommandUpdateStore()

    await update.load()

    expect(update.updating).toBe(true)
  })

  it('polls while the update runs and stops once it ends', async () => {
    fetchCommandUpdateState
      .mockResolvedValueOnce(reading('UPDATING'))
      .mockResolvedValueOnce(reading('UPDATING'))
      .mockResolvedValue(reading('UPDATED'))
    const update = useClaudeCommandUpdateStore()

    await update.load()
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MILLIS)
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MILLIS)
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MILLIS * 5)

    expect(fetchCommandUpdateState).toHaveBeenCalledTimes(3)
    expect(update.updating).toBe(false)
  })

  it('never polls when nothing is updating', async () => {
    fetchCommandUpdateState.mockResolvedValue(reading('IDLE'))

    await useClaudeCommandUpdateStore().load()
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MILLIS * 5)

    expect(fetchCommandUpdateState).toHaveBeenCalledOnce()
  })

  it('reloads the installation status when the update ends', async () => {
    fetchCommandUpdateState
      .mockResolvedValueOnce(reading('UPDATING'))
      .mockResolvedValue(reading('UPDATED'))
    await useClaudeCommandUpdateStore().load()

    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MILLIS)

    expect(fetchClaudeInstallation).toHaveBeenCalledOnce()
  })

  it('does not reload the installation when no update ever ran', async () => {
    fetchCommandUpdateState.mockResolvedValue(reading('IDLE'))

    await useClaudeCommandUpdateStore().load()

    expect(fetchClaudeInstallation).not.toHaveBeenCalled()
  })

  it('keeps polling after a reading that failed', async () => {
    fetchCommandUpdateState
      .mockResolvedValueOnce(reading('UPDATING'))
      .mockRejectedValueOnce(new Error('offline'))
      .mockResolvedValue(reading('UPDATED'))
    const update = useClaudeCommandUpdateStore()

    await update.load()
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MILLIS)
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MILLIS)

    expect(update.updating).toBe(false)
  })
})
