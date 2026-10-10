import { roleStyle } from './element-style'
import type { SheetDrawing } from './sheet-drawing'
import type { Box } from './viewport'

/** Where each element of a drawing sits: what the minimap draws and what the canvas reveals. */

export interface TintedBox extends Box {
  readonly key: string
  readonly tint: string
}

export function minimapBoxesOf(drawing: SheetDrawing): TintedBox[] {
  if (drawing.drawing === 'graph') {
    const roles = new Map(drawing.scene.nodes.map((node) => [node.id, node.role]))
    return [...drawing.layout.nodes.values()].map((placed) => ({ ...placed, key: placed.id, tint: roleStyle(roles.get(placed.id) ?? 'step').tint }))
  }
  if (drawing.drawing === 'sequence') {
    const lifelines = [...drawing.layout.participants.values()].map((placed) => ({
      key: placed.id,
      x: placed.centre - 3,
      y: placed.y,
      width: 6,
      height: placed.footY - placed.y,
      tint: roleStyle('participant').tint
    }))
    const heads = [...drawing.layout.participants.values()].map((placed) => ({ ...placed, key: `head-${placed.id}`, tint: roleStyle('participant').tint }))
    return [...lifelines, ...heads]
  }
  return []
}

/** The box an element key stands for: a card, a group, or the row a message runs along. */
export function elementBoxOf(drawing: SheetDrawing, key: string): Box | null {
  if (drawing.drawing === 'graph') return drawing.layout.nodes.get(key) ?? drawing.layout.groups.get(key) ?? null
  if (drawing.drawing !== 'sequence') return null
  if (!key.startsWith('#')) return drawing.layout.participants.get(key) ?? null
  const message = drawing.layout.messages.get(Number(key.slice(1)))
  if (!message) return null
  const left = Math.min(message.fromX, message.toX)
  return { x: left, y: message.y - 20, width: Math.max(40, Math.abs(message.toX - message.fromX)), height: 40 }
}
