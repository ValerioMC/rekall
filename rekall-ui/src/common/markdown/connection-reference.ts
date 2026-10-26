import { CONNECTION_KIND_LABEL, type ProjectConnection } from '@/features/connection/model/connection'

/** What turns a `[[...]]` reference into one naming a connection binding: `[[connection:prod]]`. */
export const CONNECTION_REFERENCE_PREFIX = 'connection:'
export const CONNECTION_CHIP_CLASS = 'rk-connection-ref'

/** The binding label a reference names, or `null` when it names a component. */
export function connectionReferenceLabel(name: string): string | null {
  const trimmed = name.trim()
  return trimmed.startsWith(CONNECTION_REFERENCE_PREFIX) ? trimmed.slice(CONNECTION_REFERENCE_PREFIX.length).trim() : null
}

/** The chip for an already escaped label, settled later by `paintConnectionReferences`. */
export function connectionChipHtml(escapedLabel: string): string {
  return `<span class="${CONNECTION_CHIP_CLASS}" data-connection="${escapedLabel}" data-state="plain">${escapedLabel}</span>`
}

function titleOf(binding: ProjectConnection): string {
  return [`${CONNECTION_KIND_LABEL[binding.kind]} · ${binding.title}`, binding.target ?? ''].filter((part) => part !== '').join('\n')
}

/**
 * Settles every connection chip under `root` against the project's bindings: found (kind, title and
 * target on hover), missing, or plain when the text has no project (`null`).
 */
export function paintConnectionReferences(root: ParentNode, connections: readonly ProjectConnection[] | null): void {
  for (const chip of root.querySelectorAll<HTMLElement>(`.${CONNECTION_CHIP_CLASS}`)) {
    const label = chip.dataset['connection'] ?? ''
    const found = connections?.find((binding) => binding.label.toLowerCase() === label.toLowerCase())
    if (connections === null) {
      chip.dataset['state'] = 'plain'
      chip.removeAttribute('title')
    } else if (found) {
      chip.dataset['state'] = 'found'
      chip.title = titleOf(found)
    } else {
      chip.dataset['state'] = 'missing'
      chip.title = `This project has no connection named “${label}”.`
    }
  }
}
