/** One screen the header switcher can take the reader to. `routes` are every route name it owns. */
export type NavScreen = Readonly<{
  to: string
  label: string
  hint: string
  routes: readonly string[]
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
