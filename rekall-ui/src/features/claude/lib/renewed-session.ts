import { fetchTerminals } from '@/features/claude/api/terminal.api'
import type { Terminal } from '@/features/claude/model/terminal'
import type { TaskId, TerminalId } from '@/model/branded'

const NEXT_SESSION_ATTEMPTS = 6
const NEXT_SESSION_DELAY_MS = 500

/**
 * After a session ends, the server may have opened the next one on the same task, when the step
 * claimed was the last of its session. Looks for it a few times; none when it never appears.
 */
export async function awaitRenewedSession(taskId: TaskId, endedId: TerminalId): Promise<Terminal | null> {
  for (let attempt = 0; attempt < NEXT_SESSION_ATTEMPTS; attempt += 1) {
    await new Promise<void>((resolve) => setTimeout(resolve, NEXT_SESSION_DELAY_MS))
    const fresh = await fetchTerminals()
    const next = fresh.find(
      (terminal) => terminal.taskId === taskId && terminal.live && terminal.id !== endedId
    )
    if (next) return next
  }
  return null
}
