import type { ComponentScan } from '@/features/project/model/project-component'

/** What a scan did, in the words the toast shows. */
export function scanSummary(scan: ComponentScan): string {
  const added = scan.added.length
  if (added === 0 && scan.alreadyKnown === 0) return 'No git repository found under the project folder'
  if (added === 0) {
    const repositories = scan.alreadyKnown === 1 ? '1 repository' : `${scan.alreadyKnown} repositories`
    return `Nothing new: ${repositories} already had a component`
  }
  const components = added === 1 ? '1 component' : `${added} components`
  return scan.alreadyKnown > 0 ? `Added ${components}, ${scan.alreadyKnown} already there` : `Added ${components}`
}
