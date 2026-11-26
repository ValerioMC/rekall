import { ApiError } from '@/api/client'
import { sessionCommandPrompt, type SessionProvider } from '@/features/claude/model/session-provider'
import { defineStore } from 'pinia'
import { computed, ref, shallowRef } from 'vue'
import {
  closeTerminal,
  fetchTerminals,
  openTerminal,
  sendTerminalInput,
  type OpenTerminalInput
} from '@/features/claude/api/terminal.api'
import { sendMessage } from '@/features/chat/api/chat.api'
import { awaitRenewedSession } from '@/features/claude/lib/renewed-session'
import type { PlanMode, RewriteMode, Terminal, TerminalTarget } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId, TerminalId } from '@/model/branded'

interface LaunchConflict {
  readonly session: Terminal
  readonly provider: SessionProvider
  readonly detail: string
  readonly decide: (retry: boolean) => void
}

interface ChatFallback {
  readonly detail: string
  readonly decide: (retry: boolean) => void
}

/**
 * In-app terminals: real PTYs running the interactive `claude` TUI, one active at a time. Never
 * persisted server-side, so the list is reloaded whenever the pane opens.
 */
export const useTerminalStore = defineStore('terminal', () => {
  const terminals = ref<Terminal[]>([])
  const activeTerminalId = ref<TerminalId | null>(null)
  const lastOpened = shallowRef<Terminal | null>(null)
  const floatingId = ref<TerminalId | null>(null)
  const launchConflict = shallowRef<LaunchConflict | null>(null)
  const chatFallback = shallowRef<ChatFallback | null>(null)

  const activeTerminal = computed(
    () => terminals.value.find((terminal) => terminal.id === activeTerminalId.value) ?? null
  )

  function terminalForTask(taskId: TaskId | null): Terminal | null {
    if (!taskId) return null
    return terminals.value.find((terminal) => terminal.taskId === taskId && terminal.live) ?? null
  }

  function liveTerminalForTask(taskId: TaskId | null): Terminal | null {
    if (!taskId) return null
    return terminals.value.find((terminal) => terminal.taskId === taskId && terminal.live && terminal.kind === 'TERMINAL') ?? null
  }

  function upsert(terminal: Terminal): void {
    const known = terminals.value.some((candidate) => candidate.id === terminal.id)
    terminals.value = known
      ? terminals.value.map((candidate) => (candidate.id === terminal.id ? terminal : candidate))
      : [terminal, ...terminals.value]
  }

  async function load(): Promise<void> {
    terminals.value = await fetchTerminals()
  }

  async function openForTask(taskId: TaskId, input: OpenTerminalInput): Promise<Terminal> {
    const opened = settleOpened(await openWithProvider({ taskId }, input))
    showInTerminalPane(opened)
    return opened
  }

  /** A terminal on a project alone, which draws a diagram, inventories or assesses; the server reuses a live one. */
  async function openForProject(projectId: ProjectId, input: OpenTerminalInput): Promise<Terminal> {
    return settleOpened(await openWithProvider({ projectId }, input))
  }

  async function planForTask(
    taskId: TaskId,
    input: Omit<OpenTerminalInput, 'stepId' | 'mode'>,
    mode: PlanMode | RewriteMode = 'PLAN',
    message = ''
  ): Promise<Terminal> {
    return openForTask(taskId, { ...input, mode, request: message.trim() || undefined })
  }

  async function openWithProvider(target: TerminalTarget, input: OpenTerminalInput): Promise<Terminal> {
    try {
      return await openTerminal(target, input)
    } catch (error) {
      if (!(error instanceof ApiError) || error.status !== 409 || launchConflict.value) throw error
      if (input.kind === 'CHAT' && input.provider === 'CODEX' && error.message.startsWith('Codex Chat could not start.')) {
        return reopenInTerminal(target, input, error)
      }
      const provider = input.provider ?? 'CLAUDE'
      const sessions = await fetchTerminals().catch(() => {
        throw error
      })
      const session = sessions.find((candidate) => blocksOpening(candidate, target, input))
      if (!session || launchConflict.value) throw error
      upsert(session)
      const retry = await new Promise<boolean>((decide) => {
        launchConflict.value = { session, provider, detail: error.message, decide }
      })
      if (!retry) throw error
      await close(session.id).catch((failure: unknown) => {
        if (!(failure instanceof ApiError) || failure.status !== 404) throw failure
      })
      return openWithProvider(target, input)
    }
  }

  async function reopenInTerminal(target: TerminalTarget, input: OpenTerminalInput, error: ApiError): Promise<Terminal> {
    if (chatFallback.value) throw error
    const retry = await new Promise<boolean>((decide) => {
      chatFallback.value = { detail: error.message, decide }
    })
    if (!retry) throw error
    return openWithProvider(target, { ...input, kind: 'TERMINAL' })
  }

  function resolveChatFallback(retry: boolean): void {
    const fallback = chatFallback.value
    chatFallback.value = null
    fallback?.decide(retry)
  }

  function blocksOpening(session: Terminal, target: TerminalTarget, input: OpenTerminalInput): boolean {
    if (!session.live) return false
    const sameTarget = 'taskId' in target
      ? session.taskId === target.taskId
      : session.taskId === null && session.projectId === target.projectId
    if (!sameTarget) return false
    const requiresKind = !input.mode || input.mode === 'WORK' || input.mode === 'MERGE'
    return session.provider !== (input.provider ?? 'CLAUDE') || (requiresKind && session.kind !== (input.kind ?? 'TERMINAL'))
  }

  function resolveLaunchConflict(retry: boolean): void {
    const conflict = launchConflict.value
    launchConflict.value = null
    conflict?.decide(retry)
  }

  /** Lists a session an open request answered with, and tells the chat window about it. */
  function settleOpened(opened: Terminal): Terminal {
    upsert(opened)
    lastOpened.value = opened
    return opened
  }

  /** A session another call opened, such as a quick task's: listed, announced and shown like one opened here. */
  function adoptOpened(opened: Terminal): void {
    settleOpened(opened)
    showInTerminalPane(opened)
  }

  /** The terminal pane shows PTY sessions only: a chat session is shown by the chat window. */
  function showInTerminalPane(opened: Terminal): void {
    if (opened.kind === 'TERMINAL') activeTerminalId.value = opened.id
  }

  /**
   * After a session ends, switches the pane to the one the server opened next on the same task,
   * when the ended one was the active one; the ended session stays listed when none appears.
   */
  async function followNextSession(taskId: TaskId, endedId: TerminalId): Promise<void> {
    const next = await awaitRenewedSession(taskId, endedId)
    if (!next) return
    upsert(next)
    if (activeTerminalId.value === endedId) activeTerminalId.value = next.id
  }

  function select(id: TerminalId | null): void {
    activeTerminalId.value = id
  }

  /** Shows a session in the floating panel above the dock instead of the full terminal pane. */
  function float(id: TerminalId): void {
    floatingId.value = id
  }

  function unfloat(): void {
    floatingId.value = null
  }

  function markEnded(id: TerminalId): void {
    terminals.value = terminals.value.map((terminal) =>
      terminal.id === id ? { ...terminal, live: false } : terminal
    )
  }

  async function close(id: TerminalId): Promise<void> {
    try {
      await closeTerminal(id)
    } finally {
      terminals.value = terminals.value.filter((terminal) => terminal.id !== id)
      if (activeTerminalId.value === id) activeTerminalId.value = null
      if (floatingId.value === id) floatingId.value = null
    }
  }

  /** Hand a line to a live session: typed into a terminal's stdin as if pasted there, sent as a message to a chat. */
  async function sendCommand(id: TerminalId, line: string): Promise<void> {
    const session = terminals.value.find((terminal) => terminal.id === id)
    if (!session?.live) throw new Error('This session has closed.')
    const prompt = sessionCommandPrompt(session.provider, line)
    if (session.kind === 'CHAT') await sendMessage(id, prompt)
    else await sendTerminalInput(id, `${prompt}\r`)
  }

  return {
    terminals,
    activeTerminalId,
    activeTerminal,
    lastOpened,
    floatingId,
    launchConflict,
    chatFallback,
    terminalForTask,
    liveTerminalForTask,
    upsert,
    load,
    openForTask,
    openForProject,
    adoptOpened,
    planForTask,
    resolveLaunchConflict,
    resolveChatFallback,
    followNextSession,
    select,
    float,
    unfloat,
    markEnded,
    close,
    sendCommand
  }
})
