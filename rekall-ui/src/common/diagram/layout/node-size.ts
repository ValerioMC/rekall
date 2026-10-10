import { textWidth, wrapText } from '../text-wrap'
import type { GraphDirection, GraphNode, NodeOutline } from '../scene/graph-scene'

/**
 * How big each element of a graph sheet is drawn and the text it carries, measured without the
 * DOM so layout stays pure. A card keeps the socket, kind label and title of every node; a class
 * or entity grows a row per member; pseudo states are small marks.
 */

export const CARD_WIDTH = 212
export const TITLE_FONT_SIZE = 12.5
export const ROW_FONT_SIZE = 11
export const ROW_HEIGHT = 18
export const BOX_HEADER = 46
export const COMPARTMENT_PADDING = 6
const ONE_LINE_HEIGHT = 56
const TWO_LINE_HEIGHT = 72
const NOTE_WIDTH = 188
const NOTE_LINE_HEIGHT = 16
const GLYPH_COLUMN = 46
const RIGHT_PADDING = 30

/** Horizontal room each outline takes from the title, beyond the glyph socket. */
const OUTLINE_INSET: Readonly<Partial<Record<NodeOutline, number>>> = { diamond: 14, hexagon: 8, slant: 14, flag: 14, stadium: 10, circle: 10, subroutine: 10 }

export interface NodeSize {
  readonly width: number
  readonly height: number
  readonly lines: readonly string[]
}

/** A fork or join bar lies across the flow, so it turns with the sheet's direction. */
export function sizeOf(node: GraphNode, direction: GraphDirection): NodeSize {
  switch (node.outline) {
    case 'start':
      return { width: 22, height: 22, lines: [] }
    case 'end':
      return { width: 26, height: 26, lines: [] }
    case 'bar':
      return direction === 'LR' || direction === 'RL' ? { width: 8, height: 80, lines: [] } : { width: 80, height: 8, lines: [] }
    case 'note':
      return noteSize(node.label)
    default:
      return node.role === 'class' || node.role === 'entity' ? boxSize(node) : cardSize(node)
  }
}

function cardSize(node: GraphNode): NodeSize {
  if (node.role === 'decision' && node.label.length === 0) return { width: 34, height: 34, lines: [] }
  const room = CARD_WIDTH - GLYPH_COLUMN - RIGHT_PADDING - (OUTLINE_INSET[node.outline] ?? 0) * 2
  const lines = wrapText(node.label, room, TITLE_FONT_SIZE, 2)
  return { width: CARD_WIDTH, height: lines.length > 1 ? TWO_LINE_HEIGHT : ONE_LINE_HEIGHT, lines }
}

function boxSize(node: GraphNode): NodeSize {
  const rows = node.compartments.flat()
  const widest = Math.max(
    textWidth(node.label, TITLE_FONT_SIZE) + GLYPH_COLUMN + 24,
    ...rows.map((row) => textWidth(row.text, ROW_FONT_SIZE) + (row.detail ? textWidth(row.detail, ROW_FONT_SIZE) + 24 : 0) + 28)
  )
  const filled = node.compartments.filter((compartment) => compartment.length > 0)
  const body = filled.reduce((sum, compartment) => sum + compartment.length * ROW_HEIGHT + COMPARTMENT_PADDING * 2, 0)
  return { width: Math.min(360, Math.max(CARD_WIDTH, Math.ceil(widest))), height: BOX_HEADER + body, lines: [node.label] }
}

function noteSize(text: string): NodeSize {
  const lines = wrapText(text, NOTE_WIDTH - 24, ROW_FONT_SIZE, 4)
  return { width: NOTE_WIDTH, height: 18 + lines.length * NOTE_LINE_HEIGHT, lines }
}
