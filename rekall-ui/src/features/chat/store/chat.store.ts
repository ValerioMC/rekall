import { defineStore } from 'pinia'
import { computed, ref, shallowRef } from 'vue'
import { answerPermission, interrupt, sendMessage } from '@/features/chat/api/chat.api'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { emptyTimeline, foldEvent } from '@/features/chat/lib/timeline'
import { awaitRenewedSession } from '@/features/claude/lib/renewed-session'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { trackSave } from '@/stores/console/rows'
import type { ChatEvent } from '@/features/chat/model/chat-event'
import type { Timeline } from '@/features/chat/model/timeline'
import type { Terminal } from '@/features/claude/model/terminal'
import type { TaskId, TerminalId } from '@/model/branded'

/**
 * Chat sessions: each one's timeline folded from its event stream, the one the window shows, and
 * the writes to it. The session records themselves stay in the terminal store, which lists both kinds.
 */
export const useChatStore = defineStore('chat', () => {
  const terminals = useTerminalStore()
  const dock = useChatDockStore()
  const timelines = shallowRef<Readonly<Record<string, Timeline>>>({})
  const activeChatId = ref<TerminalId | null>(null)
  const followingIds = ref<readonly TerminalId[]>([])
  const sendState = ref('saved')
  const answerState = ref('saved')
  const stopState = ref('saved')

  const activeTimeline = computed(() =>
    activeChatId.value ? timelineOf(activeChatId.value) : emptyTimeline()
  )
  const liveChats = computed(() =>
    terminals.terminals.filter((terminal) => terminal.kind === 'CHAT' && terminal.live)
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

  /** Brings a chat session to the screen: the window follows it and opens if it was hidden. */
  function reveal(session: Terminal): void {
    activeChatId.value = session.id
    dock.open()
  }

  function chatForTask(taskId: TaskId | null): Terminal | null {
    if (!taskId) return null
    return liveChats.value.find((terminal) => terminal.taskId === taskId) ?? null
  }

  /** Moves the window to the task's own live chat session; a task with none leaves it where it is. */
  function showTaskSession(taskId: TaskId | null): void {
    const own = chatForTask(taskId)
    if (own) activeChatId.value = own.id
  }

  function ingest(id: TerminalId, event: ChatEvent): void {
    const before = timelineOf(id)
    const after = foldEvent(before, event)
    if (after === before) return
    timelines.value = { ...timelines.value, [id]: after }
    syncAwaitingPermission(id, after)
    if (event.type === 'ended') void endSession(id)
  }

  /** The session list is fetched on load and on focus only; the event stream is what hears an approval asked or answered. */
  function syncAwaitingPermission(id: TerminalId, timeline: Timeline): void {
    const session = terminals.terminals.find((terminal) => terminal.id === id)
    const awaitingPermission = timeline.pending !== null
    if (!session || session.awaitingPermission === awaitingPermission) return
    terminals.upsert({ ...session, awaitingPermission })
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
    liveChats,
    sending,
    answering,
    stopping,
    timelineOf,
    isFollowing,
    select,
    chatForTask,
    reveal,
    showTaskSession,
    ingest,
    send,
    stop,
    allow,
    deny
  }
})
