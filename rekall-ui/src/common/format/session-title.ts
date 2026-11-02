import type { Terminal } from '@/features/claude/model/terminal'

/** What a session is listed as: its task, or what it does for a project alone. */
export function sessionTitle(terminal: Pick<Terminal, 'taskTitle' | 'projectLabel' | 'mode'>): string {
  return terminal.taskTitle ?? projectWorkTitle(terminal.mode, terminal.projectLabel)
}

function projectWorkTitle(mode: Terminal['mode'], projectLabel: string): string {
  switch (mode) {
    case 'INVENTORY':
      return `Inventory of ${projectLabel}`
    case 'ASSESS':
    case 'ASSESS_ALL':
      return `Assessment of ${projectLabel}`
    case 'AUTOCONFIGURE':
      return `Autoconfigure ${projectLabel}`
    default:
      return `Diagram for ${projectLabel}`
  }
}
