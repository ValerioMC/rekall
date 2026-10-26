import { autocompletion, type Completion } from '@codemirror/autocomplete'
import type { Extension } from '@codemirror/state'
import { CONNECTION_KIND_PATH } from '@/features/connection/lib/connection-kind-glyph'
import type { ConnectionKind } from '@/features/connection/model/connection'

/** What a `[[...]]` suggestion names, and so the glyph its row starts with. */
export type ReferenceIcon = 'component' | 'task' | ConnectionKind

interface Glyph {
  readonly path: string
  /** Drawn as a line rather than filled: the console's own marks, as opposed to a system's logo. */
  readonly outline: boolean
}

const TYPE_PREFIX = 'rk-ref-'
const SVG_NS = 'http://www.w3.org/2000/svg'

const GLYPH: Readonly<Record<ReferenceIcon, Glyph>> = {
  component: { path: 'M12 2.8 3.8 7.2v9.6l8.2 4.4 8.2-4.4V7.2L12 2.8ZM3.8 7.2 12 11.6l8.2-4.4M12 11.6v9.6', outline: true },
  task: { path: 'M6 4h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Zm2.5 8.3 2.4 2.4 4.8-5.2', outline: true },
  GITLAB: { path: CONNECTION_KIND_PATH.GITLAB, outline: false },
  KUBERNETES: { path: CONNECTION_KIND_PATH.KUBERNETES, outline: false },
  JIRA: { path: CONNECTION_KIND_PATH.JIRA, outline: false }
}

/** The completion `type` that carries an icon: CodeMirror also names the row's own icon class after it. */
export function referenceCompletionType(icon: ReferenceIcon): string {
  return `${TYPE_PREFIX}${icon.toLowerCase()}`
}

function iconOf(completion: Completion): ReferenceIcon | null {
  const icons = Object.keys(GLYPH) as ReferenceIcon[]
  return icons.find((icon) => referenceCompletionType(icon) === completion.type) ?? null
}

function svgOf(glyph: Glyph): SVGSVGElement {
  const svg = document.createElementNS(SVG_NS, 'svg')
  svg.setAttribute('viewBox', '0 0 24 24')
  svg.setAttribute('aria-hidden', 'true')
  const path = document.createElementNS(SVG_NS, 'path')
  path.setAttribute('d', glyph.path)
  if (glyph.outline) {
    path.setAttribute('fill', 'none')
    path.setAttribute('stroke', 'currentColor')
    path.setAttribute('stroke-width', '1.8')
    path.setAttribute('stroke-linecap', 'round')
    path.setAttribute('stroke-linejoin', 'round')
  } else {
    path.setAttribute('fill', 'currentColor')
  }
  svg.append(path)
  return svg
}

/** The glyph a reference suggestion starts with, or nothing for the editor's other suggestions. */
export function renderReferenceIcon(completion: Completion): Node | null {
  const icon = iconOf(completion)
  if (!icon) return null
  const holder = document.createElement('span')
  holder.className = 'rk-completion-icon'
  holder.dataset['icon'] = icon.toLowerCase()
  holder.append(svgOf(GLYPH[icon]))
  return holder
}

/** Adds the glyph to every suggestion row, merged into the editor's own completion setup. */
export function referenceCompletionIcons(): Extension {
  return autocompletion({ addToOptions: [{ render: renderReferenceIcon, position: 20 }] })
}
