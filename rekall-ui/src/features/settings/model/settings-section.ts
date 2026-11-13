export const SETTINGS_SECTIONS = [
  'general',
  'appearance',
  'review',
  'sessions',
  'notifications',
  'time',
  'connections',
  'data',
  'keyboard',
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
  about: 'About'
}

/** The section a path segment names; a missing or unknown one is General. */
export function settingsSectionOf(segment: string | undefined): SettingsSection {
  return SETTINGS_SECTIONS.find((section) => section === segment) ?? 'general'
}
