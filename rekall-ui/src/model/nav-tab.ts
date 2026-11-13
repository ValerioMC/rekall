/** Something a screen counts that needs the reader, shown beside it in the menu and summed on the key. */
export type NavAlert = 'open-vulnerabilities' | 'review-queue'

export type NavAlertTone = 'danger' | 'accent'

/**
 * One screen the header menu can take the reader to. `routes` are every route name it owns;
 * `alert` names the count it wears while it is above zero.
 */
export type NavScreen = Readonly<{
  to: string
  label: string
  hint: string
  routes: readonly string[]
  alert?: NavAlert
}>

export type NavGroup = Readonly<{
  label: string
  screens: readonly NavScreen[]
}>

export const NAV_GROUPS: readonly NavGroup[] = [
  {
    label: 'Work',
    screens: [
      {
        to: '/',
        label: 'Work',
        hint: 'Tasks, steps and sessions',
        routes: ['work', 'console'],
        alert: 'review-queue'
      }
    ]
  },
  {
    label: 'Projects',
    screens: [
      {
        to: '/projects',
        label: 'Projects',
        hint: 'Every project and its folder',
        routes: ['projects', 'project-detail']
      },
      { to: '/companies', label: 'Companies', hint: 'Who the work is for', routes: ['companies'] },
      {
        to: '/connections',
        label: 'Connections',
        hint: 'Accounts and servers shared across projects',
        routes: ['connections']
      }
    ]
  },
  {
    label: 'Time',
    screens: [
      { to: '/report', label: 'Report', hint: 'Time tracked, by period', routes: ['report'] },
      { to: '/calendar', label: 'Calendar', hint: 'Your work, day by day', routes: ['calendar'] }
    ]
  },
  {
    label: 'Library',
    screens: [
      { to: '/diagrams', label: 'Diagrams', hint: 'Generated diagrams', routes: ['diagrams'] },
      { to: '/claude', label: 'Claude files', hint: 'Instructions, memories and skills', routes: ['claude'] }
    ]
  },
  {
    label: 'Security',
    screens: [
      {
        to: '/security',
        label: 'Security',
        hint: 'Critical advisories and the projects they reach',
        routes: ['security'],
        alert: 'open-vulnerabilities'
      }
    ]
  }
]

export function isScreenActive(target: NavScreen, routeName: string | null): boolean {
  return routeName !== null && target.routes.includes(routeName)
}

export function screensOf(groups: readonly NavGroup[]): readonly NavScreen[] {
  return groups.flatMap((group) => group.screens)
}

/** The screen on display, so the menu key can wear its name; `null` when the reader is elsewhere. */
export function activeScreenIn(groups: readonly NavGroup[], routeName: string | null): NavScreen | null {
  return screensOf(groups).find((screen) => isScreenActive(screen, routeName)) ?? null
}

/** What an alert count says, read out and shown on hover. */
export function alertLabel(alert: NavAlert | undefined, count: number): string {
  if (alert === 'open-vulnerabilities') return `${count} critical ${count === 1 ? 'advisory reaches' : 'advisories reach'} your projects`
  if (alert === 'review-queue') return `${count} waiting for your review`
  return `${count} need attention`
}

/** Amber is for what waits on the reader to act; the danger tone is for a risk that exists whatever they do. */
export function alertTone(alert: NavAlert | undefined): NavAlertTone {
  return alert === 'open-vulnerabilities' ? 'danger' : 'accent'
}
