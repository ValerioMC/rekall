export type SettingsPane = 'general' | 'repository' | 'taskManager' | 'checks'

export interface SettingsPaneEntry {
  readonly pane: SettingsPane
  readonly label: string
  readonly summary: string
}

export const SETTINGS_PANES: readonly SettingsPaneEntry[] = [
  { pane: 'general', label: 'General', summary: 'Name, anchor, company, status, icon, delete' },
  { pane: 'repository', label: 'Repository', summary: 'Folder, main branch and commit on claim' },
  { pane: 'taskManager', label: 'Task manager', summary: 'Where the tasks are written' },
  { pane: 'checks', label: 'Checks', summary: 'What a claim has to pass' }
]
