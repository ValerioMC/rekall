import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  closeTerminal,
  fetchTerminals,
  openTerminal,
  sendTerminalInput,
  type OpenTerminalInput
} from '@/features/claude/api/terminal.api'
import { rkPlanCommand, rkPlanFlowCommand } from '@/common/format/rk-command'
import type { PlanMode, Terminal } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId, TerminalId } from '@/model/branded'

/**
 * In-app terminals: real PTYs running the interactive `claude` TUI, one active at a time. Never
 * persisted server-side, so the list is reloaded whenever the pane opens.
 */
export const useTerminalStore = defineStore('terminal', () => {
  const terminals = ref<Terminal[]>([])
  const activeTerminalId = ref<TerminalId | null>(null)

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
    const opened = await openTerminal({ taskId }, input)
    upsert(opened)
    activeTerminalId.value = opened.id
    return opened
  }

  /** A terminal on a project alone, which draws a diagram, inventories or assesses; the server reuses a live one. */
  async function openForProject(projectId: ProjectId, input: OpenTerminalInput): Promise<Terminal> {
    const opened = await openTerminal({ projectId }, input)
    upsert(opened)
    return opened
  }

  /**
   * Have a session propose this task's checklist, or its flow, as drafts. A live session on the
   * task gets the plan line typed into it; without one, a terminal is opened that starts on that
   * line, so a plan never needs a session already running. Either way it becomes the active one.
   */
  async function planForTask(
    taskId: TaskId,
    anchor: string,
    input: Omit<OpenTerminalInput, 'stepId' | 'mode'>,
    mode: PlanMode = 'PLAN'
  ): Promise<Terminal> {
    const live = terminalForTask(taskId)
    if (live) {
      await sendCommand(live.id, planLine(mode, anchor))
      activeTerminalId.value = live.id
      return live
    }
    const opened = await openTerminal({ taskId }, { ...input, mode })
    upsert(opened)
    activeTerminalId.value = opened.id
    return opened
  }

  function planLine(mode: PlanMode, anchor: string): string {
    return mode === 'PLAN_FLOW' ? rkPlanFlowCommand(anchor) : rkPlanCommand(anchor)
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

  /** Type a line into a live session's stdin, as if someone had pasted it there and hit enter. */
  async function sendCommand(id: TerminalId, line: string): Promise<void> {
    await sendTerminalInput(id, `${line}\r`)
  }

  return {
    terminals,
    activeTerminalId,
    activeTerminal,
    terminalForTask,
    load,
    openForTask,
    openForProject,
    planForTask,
    select,
    markEnded,
    close,
    sendCommand
  }
})
