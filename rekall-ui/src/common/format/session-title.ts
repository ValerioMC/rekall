import type { Terminal } from '@/model/terminal'

/** What a session is listed as: its task, or the diagram it generates for a project alone. */
export function sessionTitle(terminal: Pick<Terminal, 'taskTitle' | 'projectLabel'>): string {
  return terminal.taskTitle ?? `Diagram for ${terminal.projectLabel}`
}
