/** Something a screen counts that needs the reader, shown on its tab from any screen. */
export type NavAlert = 'open-vulnerabilities'

/**
 * One screen the header switcher can take the reader to. `routes` are every route name it owns;
 * `alert` names the count its tab wears while it is above zero.
 */
export type NavScreen = Readonly<{
  to: string
  label: string
  hint: string
  routes: readonly string[]
  alert?: NavAlert
}>

/** Screens that share one slot in the switcher and open from it as a menu, to save header width. */
export type NavGroup = Readonly<{
  label: string
  screens: readonly NavScreen[]
}>

export type NavEntry =
  Readonly<{ kind: 'screen'; screen: NavScreen }> | Readonly<{ kind: 'group'; group: NavGroup }>

const screen = (value: NavScreen): NavEntry => ({ kind: 'screen', screen: value })
const group = (value: NavGroup): NavEntry => ({ kind: 'group', group: value })

export const NAV_ENTRIES: readonly NavEntry[] = [
  screen({ to: '/', label: 'Console', hint: 'Tasks, steps and sessions', routes: ['console'] }),
  group({
    label: 'Catalog',
    screens: [
      {
        to: '/projects',
        label: 'Projects',
        hint: 'Every project and its folder',
        routes: ['projects', 'project-detail']
      },
      { to: '/companies', label: 'Companies', hint: 'Who the work is for', routes: ['companies'] }
    ]
  }),
  screen({ to: '/diagrams', label: 'Diagrams', hint: 'Generated diagrams', routes: ['diagrams'] }),
  screen({
    to: '/security',
    label: 'Security',
    hint: 'Critical advisories and the projects they reach',
    routes: ['security'],
    alert: 'open-vulnerabilities'
  }),
  screen({ to: '/claude', label: 'Claude', hint: 'Instructions, memories and skills', routes: ['claude'] }),
  group({
    label: 'Time',
    screens: [
      { to: '/calendar', label: 'Calendar', hint: 'Your work, day by day', routes: ['calendar'] },
      { to: '/report', label: 'Report', hint: 'Time tracked, by period', routes: ['report'] }
    ]
  })
]

export function isScreenActive(target: NavScreen, routeName: string | null): boolean {
  return routeName !== null && target.routes.includes(routeName)
}

/** The group's member on screen, so the slot can wear its name; `null` when the reader is elsewhere. */
export function activeScreenOf(target: NavGroup, routeName: string | null): NavScreen | null {
  return target.screens.find((member) => isScreenActive(member, routeName)) ?? null
}

/** The screen on display among every entry, so a collapsed switcher can wear its name. */
export function activeScreenIn(entries: readonly NavEntry[], routeName: string | null): NavScreen | null {
  for (const entry of entries) {
    const current = entry.kind === 'screen' ? (isScreenActive(entry.screen, routeName) ? entry.screen : null) : activeScreenOf(entry.group, routeName)
    if (current) return current
  }
  return null
}

/** Every screen an entry holds: the screen itself, or a group's members. */
export function screensOf(entry: NavEntry): readonly NavScreen[] {
  return entry.kind === 'screen' ? [entry.screen] : entry.group.screens
}

/** What a tab's alert count says, read out and shown on hover. */
export function alertLabel(alert: NavAlert | undefined, count: number): string {
  if (alert === 'open-vulnerabilities') return `${count} critical ${count === 1 ? 'advisory reaches' : 'advisories reach'} your projects`
  return `${count} need attention`
}
