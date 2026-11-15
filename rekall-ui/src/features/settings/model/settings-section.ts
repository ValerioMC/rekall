export const SETTINGS_SECTIONS = [
  'general',
  'connections',
  'data',
  'sessions',
  'review',
  'time',
  'notifications',
  'appearance',
  'keyboard',
  'help',
  'about'
] as const

export type SettingsSection = (typeof SETTINGS_SECTIONS)[number]

export const SETTINGS_SECTION_LABEL: Readonly<Record<SettingsSection, string>> = {
  general: 'General',
  appearance: 'Appearance',
  review: 'Review',
  sessions: 'Sessions and run queue',
  notifications: 'Notifications',
  time: 'Time and usage',
  connections: 'Connections',
  data: 'Data and backup',
  keyboard: 'Keyboard',
  help: 'Help',
  about: 'About'
}

export interface SettingsGroup {
  readonly label: string
  readonly sections: readonly SettingsSection[]
}

export const SETTINGS_GROUPS: readonly SettingsGroup[] = [
  { label: 'Workspace', sections: ['general', 'connections', 'data'] },
  { label: 'Work', sections: ['sessions', 'review', 'time', 'notifications'] },
  { label: 'Console', sections: ['appearance', 'keyboard'] },
  { label: 'Reference', sections: ['help', 'about'] }
]

/** The section a path segment names; a missing or unknown one is General. */
export function settingsSectionOf(segment: string | undefined): SettingsSection {
  return SETTINGS_SECTIONS.find((section) => section === segment) ?? 'general'
}
