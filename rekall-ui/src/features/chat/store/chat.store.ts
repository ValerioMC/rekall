import { defineStore } from 'pinia'
import { computed, ref, shallowRef } from 'vue'
import { answerPermission, interrupt, sendMessage } from '@/features/chat/api/chat.api'
import { emptyTimeline, foldEvent } from '@/features/chat/lib/timeline'
import { awaitRenewedSession } from '@/features/claude/lib/renewed-session'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { trackSave } from '@/stores/console/rows'
import type { ChatEvent } from '@/features/chat/model/chat-event'
import type { Timeline } from '@/features/chat/model/timeline'
import type { TaskId, TerminalId } from '@/model/branded'

/**
 * Chat sessions: each one's timeline folded from its event stream, the one the window shows, and
 * the writes to it. The session records themselves stay in the terminal store, which lists both kinds.
 */
export const useChatStore = defineStore('chat', () => {
  const terminals = useTerminalStore()
  const timelines = shallowRef<Readonly<Record<string, Timeline>>>({})
  const activeChatId = ref<TerminalId | null>(null)
  const followingIds = ref<readonly TerminalId[]>([])
  const sendState = ref('saved')
  const answerState = ref('saved')
  const stopState = ref('saved')

  const activeTimeline = computed(() =>
    activeChatId.value ? timelineOf(activeChatId.value) : emptyTimeline()
  )
  const sending = computed(() => sendState.value === 'saving')
  const answering = computed(() => answerState.value === 'saving')
  const stopping = computed(() => stopState.value === 'saving')

  function timelineOf(id: TerminalId): Timeline {
    return timelines.value[id] ?? emptyTimeline()
  }

  function isFollowing(id: TerminalId): boolean {
    return followingIds.value.includes(id)
  }

  function select(id: TerminalId | null): void {
    activeChatId.value = id
  }

  function ingest(id: TerminalId, event: ChatEvent): void {
    const before = timelineOf(id)
    const after = foldEvent(before, event)
    if (after === before) return
    timelines.value = { ...timelines.value, [id]: after }
    if (event.type === 'ended') void endSession(id)
  }

  async function endSession(id: TerminalId): Promise<void> {
    const session = terminals.terminals.find((terminal) => terminal.id === id)
    if (!session?.live) return
    terminals.markEnded(id)
    if (session.taskId) await followNextSession(session.taskId, id)
  }

  /**
   * Waits for the session the server opens next on the task, as a terminal does, and moves the
   * window to it when the ended one was the active one.
   */
  async function followNextSession(taskId: TaskId, endedId: TerminalId): Promise<void> {
    followingIds.value = [...followingIds.value, endedId]
    try {
      const next = await awaitRenewedSession(taskId, endedId)
      if (!next) return
      terminals.upsert(next)
      if (activeChatId.value !== endedId) return
      if (next.kind === 'CHAT') {
        activeChatId.value = next.id
        return
      }
      activeChatId.value = null
      terminals.select(next.id)
    } finally {
      followingIds.value = followingIds.value.filter((candidate) => candidate !== endedId)
    }
  }

  async function send(id: TerminalId, text: string): Promise<void> {
    await trackSave(sendState, () => sendMessage(id, text))
  }

  async function stop(id: TerminalId): Promise<void> {
    await trackSave(stopState, () => interrupt(id))
  }

  async function allow(id: TerminalId): Promise<void> {
    await answer(id, true)
  }

  async function deny(id: TerminalId): Promise<void> {
    await answer(id, false)
  }

  async function answer(id: TerminalId, allowed: boolean): Promise<void> {
    const pending = timelineOf(id).pending
    if (!pending) return
    await trackSave(answerState, () => answerPermission(id, pending.requestId, allowed))
  }

  return {
    timelines,
    activeChatId,
    activeTimeline,
    sending,
    answering,
    stopping,
    timelineOf,
    isFollowing,
    select,
    ingest,
    send,
    stop,
    allow,
    deny
  }
})
