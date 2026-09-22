import { ref } from 'vue'
import { activateDatabase, addDatabase, checkFolder } from '@/api/settings.api'
import type { AddDatabaseResult } from '@/api/settings.api'
import { env } from '@/common/config/env'
import type { DatabaseEntry, FolderCheck } from '@/model/settings'

const CHECK_DEBOUNCE_MS = 350
const HEALTH_POLL_INTERVAL_MS = 500
const HEALTH_POLL_TIMEOUT_MS = 30_000

export type SetupPhase = 'idle' | 'submitting' | 'restarting' | 'error'

export function useDatabaseSetup() {
  const checking = ref(false)
  const check = ref<FolderCheck | null>(null)
  const phase = ref<SetupPhase>('idle')
  const error = ref<string | null>(null)
  const timedOut = ref(false)

  let debounceHandle: ReturnType<typeof setTimeout> | undefined
  let requestToken = 0

  function checkPath(path: string): void {
    clearTimeout(debounceHandle)
    if (!path.trim()) {
      check.value = null
      checking.value = false
      return
    }
    checking.value = true
    const token = ++requestToken
    debounceHandle = setTimeout(async () => {
      try {
        const result = await checkFolder(path)
        if (token === requestToken) check.value = result
      } catch {
        if (token === requestToken) check.value = null
      } finally {
        if (token === requestToken) checking.value = false
      }
    }, CHECK_DEBOUNCE_MS)
  }

  async function waitForRestart(): Promise<void> {
    phase.value = 'restarting'
    if (await reloadWhenBack()) return
    phase.value = 'error'
    timedOut.value = true
    error.value = 'The application is taking longer than expected to come back.'
  }

  async function submitNewFolder(path: string, label?: string): Promise<AddDatabaseResult | null> {
    phase.value = 'submitting'
    error.value = null
    timedOut.value = false
    try {
      const result = await addDatabase({ path, label })
      await waitForRestart()
      return result
    } catch (caught) {
      phase.value = 'error'
      error.value = messageOf(caught)
      return null
    }
  }

  async function switchTo(entry: DatabaseEntry): Promise<void> {
    phase.value = 'submitting'
    error.value = null
    timedOut.value = false
    try {
      await activateDatabase(entry.id)
      await waitForRestart()
    } catch (caught) {
      phase.value = 'error'
      error.value = messageOf(caught)
    }
  }

  return { checking, check, phase, error, timedOut, checkPath, submitNewFolder, switchTo }
}

/**
 * Waits for a restarting server to answer its health check again, then reloads the page onto it.
 * Resolves false if it has not come back within the timeout, and the page stays as it is.
 */
export async function reloadWhenBack(): Promise<boolean> {
  const deadline = Date.now() + HEALTH_POLL_TIMEOUT_MS
  await sleep(HEALTH_POLL_INTERVAL_MS)
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`${env.VITE_API_BASE_URL}/actuator/health`, { cache: 'no-store' })
      if (response.ok) {
        window.location.reload()
        return true
      }
    } catch {
      // Refused while the server restarts: keep polling until the deadline.
    }
    await sleep(HEALTH_POLL_INTERVAL_MS)
  }
  return false
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : 'Something went wrong.'
}
