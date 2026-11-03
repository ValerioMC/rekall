import { watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import type { RunQueue } from '@/features/queue/model/runQueue'
import { useTerminalStore } from '@/features/claude/store/terminal.store'

/**
 * The queue opens its sessions on the server, so the console never asked for them: it reads the
 * session list again whenever the item in hand, or the step it works, changes.
 */
export function useQueueSessionSync(): void {
  const { queue } = storeToRefs(useRunQueueStore())
  const terminals = useTerminalStore()

  watch(
    () => runningSignature(queue.value),
    () => void terminals.load().catch(() => undefined),
    { immediate: true }
  )
}

function runningSignature(queue: RunQueue): string {
  return queue.items
    .filter((item) => item.state === 'RUNNING')
    .map((item) => `${item.id}:${item.steps.findIndex((step) => step.current)}`)
    .join('|')
}
