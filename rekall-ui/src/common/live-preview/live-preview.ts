import { ensureSyntaxTree, syntaxTree } from '@codemirror/language'
import {
  StateField,
  type EditorState,
  type Extension,
  type Line,
  type Range,
  type Transaction
} from '@codemirror/state'
import { Decoration, EditorView, type DecorationSet } from '@codemirror/view'
import type { SyntaxNode, SyntaxNodeRef, Tree } from '@lezer/common'
import { isAnchorRun } from '@/common/config/anchor-code'
import { BulletWidget } from '@/common/live-preview/bullet-widget'
import { CodeBlockWidget } from '@/common/live-preview/code-block-widget'
import { editorFocus, focusChanged, trackEditorFocus } from '@/common/live-preview/editor-focus'
import { ImageWidget } from '@/common/live-preview/image-widget'
import { MermaidWidget } from '@/common/live-preview/mermaid-widget'
import { referenceTargets, referenceTargetsChanged } from '@/common/live-preview/reference-targets'
import { ReferenceWidget } from '@/common/live-preview/reference-widget'
import { TableWidget } from '@/common/live-preview/table-widget'
import { TaskCheckboxWidget } from '@/common/live-preview/task-checkbox-widget'
import { referenceChipFor, NO_REFERENCE_TARGETS } from '@/common/markdown/reference-chip'

/**
 * Live preview: the markdown stays the document, byte for byte, drawn as the read view draws it.
 * Markup shows only on the caret's lines; tables, code, diagrams, images and task boxes become
 * widgets, and a block holding the caret shows its source. Unfocused, nothing shows as source.
 * A `[[...]]` naming something of the project is drawn as its chip until the caret enters it.
 */

const previewDecorations = StateField.define<DecorationSet>({
  create: (state) => decorationsFor(state),
  update: (current, transaction) =>
    needsRedraw(transaction) ? decorationsFor(transaction.state) : current.map(transaction.changes),
  provide: (field) => EditorView.decorations.from(field)
})

export function livePreview(): Extension {
  return [trackEditorFocus(), referenceTargets, previewDecorations]
}

const MARKS_HIDDEN_WITH_SPACE = new Set(['HeaderMark', 'QuoteMark'])
const MARKS_HIDDEN = new Set(['EmphasisMark', 'StrikethroughMark', 'CodeMark'])
const INLINE_STYLES: Record<string, string> = {
  Emphasis: 'cm-rk-em',
  StrongEmphasis: 'cm-rk-strong',
  Strikethrough: 'cm-rk-strike'
}
// How long to wait for the parser on a rebuild before drawing from the part already parsed.
const PARSE_BUDGET_MS = 50
const REFERENCE = /\[\[(?!\[)(.*?)\]\]/g
const NOT_REFERENCED_IN = new Set(['FencedCode', 'CodeBlock', 'InlineCode', 'Table'])

/** One rebuild's inputs and output: the state, the lines the caret is on, the ranges so far. */
interface Scan {
  state: EditorState
  focused: boolean
  caretLines: Set<number>
  ranges: Range<Decoration>[]
  references: ReferenceSpan[]
}

interface ReferenceSpan {
  from: number
  to: number
}

function needsRedraw(transaction: Transaction): boolean {
  return (
    transaction.docChanged ||
    transaction.selection !== undefined ||
    transaction.effects.some((effect) => effect.is(focusChanged) || effect.is(referenceTargetsChanged)) ||
    syntaxTree(transaction.state) !== syntaxTree(transaction.startState)
  )
}

function decorationsFor(state: EditorState): DecorationSet {
  const focused = state.field(editorFocus, false) ?? false
  const scan: Scan = {
    state,
    focused,
    caretLines: focused ? caretLinesOf(state) : new Set(),
    ranges: [],
    references: []
  }
  const tree = ensureSyntaxTree(state, state.doc.length, PARSE_BUDGET_MS) ?? syntaxTree(state)
  collectReferences(scan, tree)
  tree.iterate({ enter: (node) => decorate(scan, node) })
  return Decoration.set(scan.ranges, true)
}

function caretLinesOf(state: EditorState): Set<number> {
  const lines = new Set<number>()
  for (const range of state.selection.ranges) {
    const last = state.doc.lineAt(range.to).number
    for (let line = state.doc.lineAt(range.from).number; line <= last; line++) lines.add(line)
  }
  return lines
}

function collectReferences(scan: Scan, tree: Tree): void {
  const targets = scan.state.field(referenceTargets, false) ?? NO_REFERENCE_TARGETS
  for (let number = 1; number <= scan.state.doc.lines; number++) {
    const line = scan.state.doc.line(number)
    for (const match of line.text.matchAll(REFERENCE)) {
      const written = match[1] ?? ''
      const from = line.from + match.index
      if (written.trim() === '' || isInsideCode(tree, from)) continue
      const reference = { from, to: from + match[0].length }
      scan.references.push(reference)
      const chip = referenceChipFor(written, targets)
      if (chip && !caretInReference(scan, reference))
        scan.ranges.push(Decoration.replace({ widget: new ReferenceWidget(match[0], chip) }).range(from, reference.to))
    }
  }
}

function isInsideCode(tree: Tree, position: number): boolean {
  for (let node: SyntaxNode | null = tree.resolveInner(position, 1); node; node = node.parent) {
    if (NOT_REFERENCED_IN.has(node.name)) return true
  }
  return false
}

/** Returns false where the node's children must not be decorated: they are inside a widget or shown raw. */
function decorate(scan: Scan, node: SyntaxNodeRef): boolean | void {
  const name = node.name
  if (name === 'FencedCode') return decorateFencedCode(scan, node.node)
  if (name === 'Table') return decorateTable(scan, node.node)
  if (name === 'Image') return decorateImage(scan, node.node)
  if (name.startsWith('ATXHeading') || name.startsWith('SetextHeading'))
    styleLines(scan, node, `cm-rk-heading cm-rk-h${node.name.slice(-1)}`)
  else if (name === 'Blockquote') styleLines(scan, node, 'cm-rk-quote')
  else if (name === 'HorizontalRule') decorateRule(scan, node)
  else if (name === 'InlineCode') styleInlineCode(scan, node)
  else if ((name === 'Link' || name === 'Autolink') && overlapsReference(scan, node)) return false
  else if (name === 'Link' || name === 'Autolink') decorateLink(scan, node.node)
  else if (name === 'URL' && !isInsideLink(node.node))
    scan.ranges.push(Decoration.mark({ class: 'cm-rk-link' }).range(node.from, node.to))
  else if (name === 'ListMark') decorateListMark(scan, node.node)
  else if (name === 'TaskMarker') decorateTaskMarker(scan, node)
  else if (INLINE_STYLES[name])
    scan.ranges.push(Decoration.mark({ class: INLINE_STYLES[name] }).range(node.from, node.to))
  else if (MARKS_HIDDEN_WITH_SPACE.has(name)) hideMark(scan, node, true)
  else if (MARKS_HIDDEN.has(name)) hideMark(scan, node, false)
}

function decorateFencedCode(scan: Scan, node: SyntaxNode): false {
  const { first, last } = blockLines(scan, node)
  if (holdsCaret(scan, first.from, last.to)) {
    styleLines(scan, node, 'cm-rk-codeline')
    return false
  }
  const info = node.getChild('CodeInfo')
  const language = info ? slice(scan, info).trim().toLowerCase() : ''
  const text = node.getChild('CodeText')
  const code = text ? slice(scan, text) : ''
  const widget = language === 'mermaid' ? new MermaidWidget(code) : new CodeBlockWidget(code, language)
  scan.ranges.push(Decoration.replace({ widget, block: true }).range(first.from, last.to))
  return false
}

function decorateTable(scan: Scan, node: SyntaxNode): false {
  const { first, last } = blockLines(scan, node)
  if (holdsCaret(scan, first.from, last.to)) return false
  scan.ranges.push(
    Decoration.replace({ widget: new TableWidget(slice(scan, node)), block: true }).range(first.from, last.to)
  )
  return false
}

function decorateImage(scan: Scan, node: SyntaxNode): false {
  if (onCaretLine(scan, node.from)) return false
  const url = node.getChild('URL')
  const altEnd = node.getChildren('LinkMark')[1]?.from ?? node.from + 2
  const alt = scan.state.sliceDoc(node.from + 2, altEnd)
  scan.ranges.push(
    Decoration.replace({ widget: new ImageWidget(url ? slice(scan, url) : '', alt) }).range(
      node.from,
      node.to
    )
  )
  return false
}

function decorateRule(scan: Scan, node: SyntaxNodeRef): void {
  styleLines(scan, node, 'cm-rk-hr')
  if (!onCaretLine(scan, node.from)) scan.ranges.push(Decoration.replace({}).range(node.from, node.to))
}

function styleInlineCode(scan: Scan, node: SyntaxNodeRef): void {
  const content = slice(scan, node).replace(/^`+|`+$/g, '')
  const style = isAnchorRun(content) ? 'cm-rk-code rk-anchor' : 'cm-rk-code'
  scan.ranges.push(Decoration.mark({ class: style }).range(node.from, node.to))
}

/** Off the caret's line only the link text is left: `[`, and everything from `]` on, are hidden. */
function decorateLink(scan: Scan, node: SyntaxNode): void {
  const marks = node.getChildren('LinkMark')
  const opening = marks[0]
  const closing = marks[1]
  const textFrom = opening ? opening.to : node.from
  const textTo = closing ? closing.from : node.to
  if (textTo > textFrom) scan.ranges.push(Decoration.mark({ class: 'cm-rk-link' }).range(textFrom, textTo))
  if (onCaretLine(scan, node.from)) return
  if (opening) scan.ranges.push(Decoration.replace({}).range(opening.from, opening.to))
  if (closing && closing.from < node.to) scan.ranges.push(Decoration.replace({}).range(closing.from, node.to))
}

/** A bullet becomes the read view's dash, or nothing when a checkbox stands in for it; numbers stay. */
function decorateListMark(scan: Scan, node: SyntaxNode): void {
  const item = node.parent
  if (item?.parent?.name === 'OrderedList') {
    scan.ranges.push(Decoration.mark({ class: 'cm-rk-list-number' }).range(node.from, node.to))
    return
  }
  if (onCaretLine(scan, node.from)) return
  if (item?.getChild('Task')) hideMark(scan, node, true)
  else scan.ranges.push(Decoration.replace({ widget: new BulletWidget() }).range(node.from, node.to))
}

function decorateTaskMarker(scan: Scan, node: SyntaxNodeRef): void {
  if (onCaretLine(scan, node.from)) return
  const checked = /x/i.test(slice(scan, node))
  scan.ranges.push(Decoration.replace({ widget: new TaskCheckboxWidget(checked) }).range(node.from, node.to))
}

function hideMark(scan: Scan, node: SyntaxNodeRef, withSpace: boolean): void {
  if (onCaretLine(scan, node.from)) return
  const to = withSpace && scan.state.sliceDoc(node.to, node.to + 1) === ' ' ? node.to + 1 : node.to
  if (to > node.from) scan.ranges.push(Decoration.replace({}).range(node.from, to))
}

function styleLines(scan: Scan, node: SyntaxNodeRef, style: string): void {
  const { first, last } = blockLines(scan, node)
  for (let number = first.number; number <= last.number; number++) {
    scan.ranges.push(Decoration.line({ class: style }).range(scan.state.doc.line(number).from))
  }
}

function blockLines(scan: Scan, node: SyntaxNodeRef): { first: Line; last: Line } {
  return { first: scan.state.doc.lineAt(node.from), last: scan.state.doc.lineAt(node.to) }
}

function holdsCaret(scan: Scan, from: number, to: number): boolean {
  return scan.focused && scan.state.selection.ranges.some((range) => range.from <= to && range.to >= from)
}

/** Source shows with the caret inside a reference or before it; just after it the chip stays, so one just completed is drawn at once. */
function caretInReference(scan: Scan, reference: ReferenceSpan): boolean {
  return (
    scan.focused &&
    scan.state.selection.ranges.some(
      (range) => range.from < reference.to && (range.to > reference.from || range.from === reference.from)
    )
  )
}

function overlapsReference(scan: Scan, node: SyntaxNodeRef): boolean {
  return scan.references.some((reference) => node.from < reference.to && node.to > reference.from)
}

function onCaretLine(scan: Scan, position: number): boolean {
  return scan.caretLines.has(scan.state.doc.lineAt(position).number)
}

function isInsideLink(node: SyntaxNode): boolean {
  const parent = node.parent?.name
  return parent === 'Link' || parent === 'Image' || parent === 'Autolink'
}

function slice(scan: Scan, node: { from: number; to: number }): string {
  return scan.state.sliceDoc(node.from, node.to)
}
