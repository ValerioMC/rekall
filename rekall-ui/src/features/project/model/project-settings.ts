export type SettingsPane = 'general' | 'repository' | 'checks'

export interface SettingsPaneEntry {
  readonly pane: SettingsPane
  readonly label: string
  readonly summary: string
}

export const SETTINGS_PANES: readonly SettingsPaneEntry[] = [
  { pane: 'general', label: 'General', summary: 'Name, anchor, company, status, icon, delete' },
  { pane: 'repository', label: 'Repository', summary: 'Folder and commit on claim' },
  { pane: 'checks', label: 'Checks', summary: 'What a claim has to pass' }
]
