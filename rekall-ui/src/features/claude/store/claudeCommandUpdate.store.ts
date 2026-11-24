import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { fetchCommandUpdateState } from '@/features/claude/api/claude.api'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import type { CommandUpdateState } from '@/features/claude/model/claude'

const POLL_INTERVAL_MILLIS = 1000

/**
 * The server replaces a stale /rk command by itself at startup. This follows that update while
 * it runs and reloads the installation once it ends, so the badge and the Settings key reflect
 * the new files.
 */
export const useClaudeCommandUpdateStore = defineStore('claudeCommandUpdate', () => {
  const connection = useClaudeConnectionStore()
  const commandUpdate = ref<CommandUpdateState>({ state: 'IDLE' })
  const updating = computed<boolean>(() => commandUpdate.value.state === 'UPDATING')
  let pollTimer: ReturnType<typeof setTimeout> | null = null

  async function load(): Promise<void> {
    const wasUpdating = updating.value
    commandUpdate.value = await fetchCommandUpdateState()
    if (updating.value) return schedulePoll()
    if (wasUpdating) await connection.load()
  }

  function schedulePoll(): void {
    if (pollTimer) return
    pollTimer = setTimeout(poll, POLL_INTERVAL_MILLIS)
  }

  async function poll(): Promise<void> {
    pollTimer = null
    await load().catch(schedulePoll)
  }

  return { commandUpdate, updating, load }
})
