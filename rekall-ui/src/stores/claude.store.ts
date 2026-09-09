import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  clearClaudeSession,
  deleteClaudeSession,
  fetchClaudeSessions,
  fetchClaudeTranscript,
  fetchTaskClaudeSessions,
  sendClaudePrompt,
  startClaudeSession,
  stopClaudeSession,
  type StartClaudeSessionInput
} from '@/api/claude.api'
import type { ClaudeMessage, ClaudeSession } from '@/model/claude'
import type { ClaudeSessionId, TaskId } from '@/model/branded'

/**
 * Hosted Claude sessions and their transcripts. One session is "active" at a time: the one the
 * session pane is showing, and the only one the live feed follows. Everything else is a list of
 * status chips, refreshed when the pane opens or an action lands.
 */
export const useClaudeStore = defineStore('claude', () => {
  const sessions = ref<ClaudeSession[]>([])
  const messages = ref<Record<string, ClaudeMessage[]>>({})
  const activeSessionId = ref<ClaudeSessionId | null>(null)

  const liveSessions = computed(() => sessions.value.filter((session) => session.live))

  const activeSession = computed(
    () => sessions.value.find((session) => session.id === activeSessionId.value) ?? null
  )

  const activeMessages = computed(() =>
    activeSessionId.value ? (messages.value[activeSessionId.value] ?? []) : []
  )

  function sessionsForTask(taskId: TaskId | null): ClaudeSession[] {
    if (!taskId) return []
    return sessions.value
      .filter((session) => session.taskId === taskId)
      .sort((a, b) => b.startedAt.localeCompare(a.startedAt))
  }

  function upsertSession(session: ClaudeSession): void {
    const known = sessions.value.some((candidate) => candidate.id === session.id)
    sessions.value = known
      ? sessions.value.map((candidate) => (candidate.id === session.id ? session : candidate))
      : [session, ...sessions.value]
  }

  async function loadSessions(): Promise<void> {
    sessions.value = await fetchClaudeSessions()
  }

  async function loadTaskSessions(taskId: TaskId): Promise<void> {
    const own = await fetchTaskClaudeSessions(taskId)
    const others = sessions.value.filter((session) => session.taskId !== taskId)
    sessions.value = [...own, ...others]
  }

  async function openTranscript(sessionId: ClaudeSessionId): Promise<void> {
    const transcript = await fetchClaudeTranscript(sessionId)
    messages.value = { ...messages.value, [sessionId]: transcript }
  }

  async function selectSession(sessionId: ClaudeSessionId | null): Promise<void> {
    activeSessionId.value = sessionId
    if (sessionId && !messages.value[sessionId]) {
      await openTranscript(sessionId)
    }
  }

  /**
   * One live session per task. A task-level press (no step) with a session already running just
   * focuses it. A step-level press always goes to the server, which reuses the same process and
   * retargets the step rather than starting a second one.
   */
  async function startForTask(
    taskId: TaskId,
    input: StartClaudeSessionInput
  ): Promise<ClaudeSession> {
    if (!input.stepId) {
      const live = sessions.value.find((session) => session.taskId === taskId && session.live)
      if (live) {
        await selectSession(live.id)
        return live
      }
    }
    const created = await startClaudeSession(taskId, input)
    upsertSession(created)
    if (!messages.value[created.id]) {
      messages.value = { ...messages.value, [created.id]: [] }
    }
    await selectSession(created.id)
    return created
  }

  async function clearSession(sessionId: ClaudeSessionId): Promise<void> {
    upsertSession(await clearClaudeSession(sessionId))
  }

  async function sendPrompt(sessionId: ClaudeSessionId, text: string): Promise<void> {
    await sendClaudePrompt(sessionId, text)
  }

  async function stop(sessionId: ClaudeSessionId): Promise<void> {
    upsertSession(await stopClaudeSession(sessionId))
  }

  async function remove(sessionId: ClaudeSessionId): Promise<void> {
    await deleteClaudeSession(sessionId)
    sessions.value = sessions.value.filter((session) => session.id !== sessionId)
    const rest = { ...messages.value }
    delete rest[sessionId]
    messages.value = rest
    if (activeSessionId.value === sessionId) activeSessionId.value = null
  }

  function applyMessage(message: ClaudeMessage): void {
    const current = messages.value[message.sessionId] ?? []
    if (current.some((existing) => existing.id === message.id)) return
    messages.value = {
      ...messages.value,
      [message.sessionId]: [...current, message].sort((a, b) => a.seq - b.seq)
    }
  }

  function applySession(session: ClaudeSession): void {
    upsertSession(session)
  }

  return {
    sessions,
    messages,
    activeSessionId,
    liveSessions,
    activeSession,
    activeMessages,
    sessionsForTask,
    loadSessions,
    loadTaskSessions,
    openTranscript,
    selectSession,
    startForTask,
    clearSession,
    sendPrompt,
    stop,
    remove,
    applyMessage,
    applySession,
    upsertSession
  }
})
