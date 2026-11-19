export const PROJECT_TABS = [
  'tasks',
  'blueprint',
  'pages',
  'components',
  'diagrams',
  'deployment',
  'connections',
  'settings'
] as const

export type ProjectTab = (typeof PROJECT_TABS)[number]

export const PROJECT_TAB_LABEL: Readonly<Record<ProjectTab, string>> = {
  tasks: 'Tasks',
  blueprint: 'Blueprint',
  pages: 'Pages',
  components: 'Components',
  diagrams: 'Diagrams',
  deployment: 'Deployment',
  connections: 'Connections',
  settings: 'Settings'
}

/** The tab a path segment names; a missing or unknown one is the tasks. */
export function projectTabOf(segment: string | undefined): ProjectTab {
  return PROJECT_TABS.find((tab) => tab === segment) ?? 'tasks'
}
