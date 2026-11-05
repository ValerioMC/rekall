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
import { rkPlanCommand, rkPlanFlowCommand, rkReplanCommand, rkReviseCommand } from '@/common/format/rk-command'
import type { PlanMode, Terminal } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId, TerminalId } from '@/model/branded'

/**
 * In-app terminals: real PTYs running the interactive `claude` TUI, one active at a time. Never
 * persisted server-side, so the list is reloaded whenever the pane opens.
 */
export const useTerminalStore = defineStore('terminal', () => {
  const terminals = ref<Terminal[]>([])
  const activeTerminalId = ref<TerminalId | null>(null)
  const lastOpened = shallowRef<Terminal | null>(null)

  const activeTerminal = computed(
    () => terminals.value.find((terminal) => terminal.id === activeTerminalId.value) ?? null
  )

  function terminalForTask(taskId: TaskId | null): Terminal | null {
    if (!taskId) return null
    return terminals.value.find((terminal) => terminal.taskId === taskId && terminal.live) ?? null
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

  // One live terminal per task: the server reuses a live one, retargets it and types the `/rk`
  // line into it, so the session reloads the task's latest steps instead of sitting idle.
  async function openForTask(taskId: TaskId, input: OpenTerminalInput): Promise<Terminal> {
    const opened = settleOpened(await openTerminal({ taskId }, input))
    showInTerminalPane(opened)
    return opened
  }

  /** A terminal on a project alone, which draws a diagram, inventories or assesses; the server reuses a live one. */
  async function openForProject(projectId: ProjectId, input: OpenTerminalInput): Promise<Terminal> {
    return settleOpened(await openTerminal({ projectId }, input))
  }

  /**
   * Have a session propose this task's checklist, or its flow, or redesign the steps it has
   * (`REPLAN`), bending them to a message about what the plan missed, or evaluate the pending
   * entries of its thread (`REVISE`). A live session on the
   * task gets the plan line typed into it; without one, a terminal is opened that starts on that
   * line, so a plan never needs a session already running. Either way it becomes the active one.
   */
  async function planForTask(
    taskId: TaskId,
    anchor: string,
    input: Omit<OpenTerminalInput, 'stepId' | 'mode'>,
    mode: PlanMode = 'PLAN',
    message = ''
  ): Promise<Terminal> {
    const live = terminalForTask(taskId)
    if (live) {
      await sendCommand(live.id, planLine(mode, anchor, message))
      showInTerminalPane(live)
      lastOpened.value = live
      return live
    }
    const opened = settleOpened(await openTerminal({ taskId }, { ...input, mode, request: message.trim() || undefined }))
    showInTerminalPane(opened)
    return opened
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

  function planLine(mode: PlanMode, anchor: string, message: string): string {
    if (mode === 'PLAN_FLOW') return rkPlanFlowCommand(anchor)
    if (mode === 'REVISE') return rkReviseCommand(anchor)
    return mode === 'REPLAN' ? rkReplanCommand(anchor, message) : rkPlanCommand(anchor)
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
    }
  }

  /** Hand a line to a live session: typed into a terminal's stdin as if pasted there, sent as a message to a chat. */
  async function sendCommand(id: TerminalId, line: string): Promise<void> {
    const isChat = terminals.value.some((terminal) => terminal.id === id && terminal.kind === 'CHAT')
    if (isChat) await sendMessage(id, line)
    else await sendTerminalInput(id, `${line}\r`)
  }

  return {
    terminals,
    activeTerminalId,
    activeTerminal,
    lastOpened,
    terminalForTask,
    upsert,
    load,
    openForTask,
    openForProject,
    adoptOpened,
    planForTask,
    followNextSession,
    select,
    markEnded,
    close,
    sendCommand
  }
})
