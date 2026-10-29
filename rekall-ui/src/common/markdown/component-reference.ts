import type { config } from 'md-editor-v3'
import type { ProjectComponent } from '@/features/project/model/project-component'
import { connectionChipHtml, connectionReferenceLabel } from '@/common/markdown/connection-reference'
import { taskChipHtml, taskReferenceLabel } from '@/common/markdown/task-reference'

type MarkdownIt = Parameters<NonNullable<Parameters<typeof config>[0]['markdownItConfig']>>[0]
type InlineRule = Parameters<MarkdownIt['inline']['ruler']['before']>[2]

const TOKEN = 'rk_component_ref'
export const CHIP_CLASS = 'rk-component-ref'
const DESCRIPTION_TITLE_MAX = 280

/** How a name is compared with a component's: ignoring case and surrounding spaces, as the backend does. */
export function componentNameKey(name: string): string {
  return name.trim().toLowerCase()
}

/**
 * `[[Name]]` on one line, outside code (code spans are consumed by markdown-it before this rule
 * sees them, fenced blocks never reach the inline rules). A blank name, one starting with `[`, or
 * no closing `]]` is plain text, the same edge cases as the backend's expansion.
 */
const referenceRule: InlineRule = (state, silent) => {
  const { src, pos, posMax } = state
  if (!src.startsWith('[[', pos)) return false
  const close = src.indexOf(']]', pos + 2)
  if (close === -1 || close >= posMax) return false
  const name = src.slice(pos + 2, close)
  if (name.includes('\n') || name.trim() === '' || name.startsWith('[')) return false
  if (!silent) state.push(TOKEN, '', 0).content = name
  state.pos = close + 2
  return true
}

/**
 * Renders a reference as a chip carrying the written name, or the label of a `[[connection:label]]`
 * or `[[task:label]]`. The page does not know which project the text belongs to, so
 * `paintComponentReferences`, `paintConnectionReferences` and `paintTaskReferences` settle each chip once it does.
 */
export function markComponentReferences(md: MarkdownIt): void {
  md.inline.ruler.before('link', TOKEN, referenceRule)
  md.renderer.rules[TOKEN] = (tokens, index) => {
    const written = tokens[index]?.content ?? ''
    const label = connectionReferenceLabel(written)
    if (label !== null) return connectionChipHtml(md.utils.escapeHtml(label))
    const task = taskReferenceLabel(written)
    if (task !== null) return taskChipHtml(md.utils.escapeHtml(task))
    const name = md.utils.escapeHtml(written)
    return `<span class="${CHIP_CLASS}" data-component="${name}" data-state="plain">${name}</span>`
  }
}

export function componentTitle(component: ProjectComponent): string {
  const description = component.description.trim()
  const shown = description.length > DESCRIPTION_TITLE_MAX ? `${description.slice(0, DESCRIPTION_TITLE_MAX)}…` : description
  return [component.folder, shown].filter((part) => part !== '').join('\n\n')
}

export function missingComponentTitle(name: string): string {
  return `This project has no component named “${name.trim()}”.`
}

/**
 * Settles every chip under `root` against the project's components: found (folder and description
 * on hover), missing (a broken reference), or plain when the text has no project (`null`).
 */
export function paintComponentReferences(root: ParentNode, components: readonly ProjectComponent[] | null): void {
  for (const chip of root.querySelectorAll<HTMLElement>(`.${CHIP_CLASS}`)) {
    const name = chip.dataset['component'] ?? ''
    const found = components?.find((component) => componentNameKey(component.name) === componentNameKey(name))
    if (components === null) {
      chip.dataset['state'] = 'plain'
      chip.removeAttribute('title')
    } else if (found) {
      chip.dataset['state'] = 'found'
      chip.title = componentTitle(found)
    } else {
      chip.dataset['state'] = 'missing'
      chip.title = missingComponentTitle(name)
    }
  }
}
