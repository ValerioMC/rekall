/** What turns a `[[...]]` reference into one naming another page of the project: `[[page:Deployment]]`. */
export const PAGE_REFERENCE_PREFIX = 'page:'
export const PAGE_CHIP_CLASS = 'rk-page-ref'

/** What a reference needs to know of a page: its title, the context it is kept in, and how long it is. */
export interface ReferencePage {
  readonly title: string
  readonly contextTitle: string
  readonly characters: number
}

/** The page title a reference names, or `null` when it names something else. */
export function pageReferenceTitle(name: string): string | null {
  const trimmed = name.trim()
  return trimmed.startsWith(PAGE_REFERENCE_PREFIX) ? trimmed.slice(PAGE_REFERENCE_PREFIX.length).trim() : null
}

/** How a title is compared with a page's: ignoring case and surrounding spaces, as the backend does. */
export function pageTitleKey(title: string): string {
  return title.trim().toLowerCase()
}

/** The chip for an already escaped title, settled later by `paintPageReferences`. */
export function pageChipHtml(escapedTitle: string): string {
  return `<span class="${PAGE_CHIP_CLASS}" data-page="${escapedTitle}" data-state="plain">${escapedTitle}</span>`
}

export function pageTitle(page: ReferencePage): string {
  return `${page.contextTitle}\n${page.title}\n${page.characters.toLocaleString('en-US')} characters`
}

export function missingPageTitle(title: string): string {
  return `This project has no page titled “${title}”.`
}

/** The page a reference names, or `undefined` when the project has none by that title. */
export function pageNamed(pages: readonly ReferencePage[], title: string): ReferencePage | undefined {
  return pages.find((page) => pageTitleKey(page.title) === pageTitleKey(title))
}

/**
 * Settles every page chip under `root` against the project's pages: found (context, title and size
 * on hover, and a click opens it), missing, or plain when the text has no project (`null`).
 */
export function paintPageReferences(root: ParentNode, pages: readonly ReferencePage[] | null): void {
  for (const chip of root.querySelectorAll<HTMLElement>(`.${PAGE_CHIP_CLASS}`)) {
    const title = chip.dataset['page'] ?? ''
    const found = pages === null ? undefined : pageNamed(pages, title)
    if (pages === null) {
      chip.dataset['state'] = 'plain'
      chip.removeAttribute('title')
    } else if (found) {
      chip.dataset['state'] = 'found'
      chip.title = pageTitle(found)
    } else {
      chip.dataset['state'] = 'missing'
      chip.title = missingPageTitle(title)
    }
  }
}
