import { textWidth, wrapText } from '../text-wrap'
import { union } from './geometry'
import type { FragmentKind, SequenceMessage, SequenceScene, SequenceStep } from '../scene/sequence-scene'
import type { Box, Point } from '../viewport'

/**
 * Geometry for a sequence sheet: a column per participant, wide enough for the messages that
 * cross it, then a row per step from the top. Pure and deterministic, like the graph layout.
 */

export const PARTICIPANT_HEIGHT = 56
export const ACTIVATION_WIDTH = 10
const PARTICIPANT_MIN_WIDTH = 168
const PARTICIPANT_MAX_WIDTH = 280
const PARTICIPANT_GAP = 56
const TITLE_FONT_SIZE = 12.5
const LABEL_FONT_SIZE = 11.5
const NOTE_FONT_SIZE = 11
const NOTE_WIDTH = 168
const NOTE_LINE = 16
const MESSAGE_LABEL_ROOM = 22
const MESSAGE_ROW = 22
const SELF_LOOP_HEIGHT = 26
const SELF_LOOP_WIDTH = 34
const FRAGMENT_HEADER = 30
const FRAGMENT_SECTION = 26
const FRAGMENT_PADDING = 14
const FRAGMENT_INSET = 8
const ACTIVATION_NEST = 5
const BOX_PADDING = 12
const BOX_HEADER = 24

export interface PlacedParticipant extends Box {
  readonly id: string
  readonly centre: number
  /** Where the copy of the header at the foot of the lifeline starts. */
  readonly footY: number
}

export interface PlacedMessage {
  readonly number: number
  readonly fromX: number
  readonly toX: number
  readonly y: number
  readonly self: boolean
  readonly labelAt: Point
}

export interface PlacedNote extends Box {
  readonly index: number
  readonly lines: readonly string[]
}

export interface PlacedFragment extends Box {
  readonly kind: FragmentKind
  readonly label: string
  readonly depth: number
  readonly sections: readonly { readonly label: string; readonly y: number }[]
}

export interface PlacedActivation extends Box {
  readonly participantId: string
}

export interface PlacedBox extends Box {
  readonly label: string | null
}

export interface SequenceLayout {
  readonly participants: ReadonlyMap<string, PlacedParticipant>
  readonly messages: ReadonlyMap<number, PlacedMessage>
  readonly notes: readonly PlacedNote[]
  readonly fragments: readonly PlacedFragment[]
  readonly activations: readonly PlacedActivation[]
  readonly boxes: readonly PlacedBox[]
  readonly lifelineTop: number
  readonly lifelineBottom: number
  readonly bounds: Box
}

interface OpenFragment {
  readonly kind: FragmentKind
  readonly label: string
  readonly top: number
  readonly depth: number
  readonly sections: { label: string; y: number }[]
  readonly touched: Set<string>
}

export function participantWidth(label: string): number {
  return Math.min(PARTICIPANT_MAX_WIDTH, Math.max(PARTICIPANT_MIN_WIDTH, Math.ceil(textWidth(label, TITLE_FONT_SIZE) + 76)))
}

export function layoutSequence(scene: SequenceScene): SequenceLayout {
  const centres = columnCentres(scene)
  const widths = new Map(scene.participants.map((participant) => [participant.id, participantWidth(participant.label)]))
  const order = new Map(scene.participants.map((participant, index) => [participant.id, index]))
  const boxTop = scene.boxes.length > 0 ? BOX_HEADER + BOX_PADDING : 0
  const lifelineTop = boxTop + PARTICIPANT_HEIGHT

  const messages = new Map<number, PlacedMessage>()
  const notes: PlacedNote[] = []
  const fragments: PlacedFragment[] = []
  const activations: PlacedActivation[] = []
  const openActivations = new Map<string, number[]>()
  const openFragments: OpenFragment[] = []
  let y = lifelineTop + 26
  let lastMessage: { readonly number: number; readonly y: number } | null = null

  const depthOf = (id: string): number => openActivations.get(id)?.length ?? 0
  const touch = (...ids: string[]): void => openFragments.forEach((fragment) => ids.forEach((id) => fragment.touched.add(id)))
  const centreOf = (id: string): number => centres.get(id) ?? 0

  scene.steps.forEach((step, index) => {
    switch (step.type) {
      case 'message': {
        touch(step.from, step.to)
        y += MESSAGE_LABEL_ROOM
        messages.set(step.number, placeMessage(step, y, centreOf, depthOf, order))
        lastMessage = { number: step.number, y }
        y += step.from === step.to ? SELF_LOOP_HEIGHT + MESSAGE_ROW : MESSAGE_ROW
        break
      }
      case 'activate': {
        const starts = openActivations.get(step.participantId) ?? []
        const lastY = lastMessage?.y
        openActivations.set(step.participantId, [...starts, lastY !== undefined && previousIsMessage(scene.steps, index) ? lastY : y])
        break
      }
      case 'deactivate': {
        const starts = openActivations.get(step.participantId) ?? []
        const start = starts.pop()
        if (start === undefined) break
        activations.push(activationBar(step.participantId, starts.length, start, Math.max(y - MESSAGE_ROW / 2, start + 12), centreOf))
        break
      }
      case 'note': {
        touch(...step.participantIds)
        const placed = placeNote(index, step.placement, step.participantIds, step.text, y + 6, centreOf)
        notes.push(placed)
        y = placed.y + placed.height + 12
        break
      }
      case 'fragment-start':
        y += 10
        openFragments.push({ kind: step.kind, label: step.label, top: y, depth: openFragments.length, sections: [], touched: new Set() })
        y += FRAGMENT_HEADER
        break
      case 'fragment-section': {
        const open = openFragments[openFragments.length - 1]
        if (!open) break
        y += 6
        open.sections.push({ label: step.label, y })
        y += FRAGMENT_SECTION
        break
      }
      case 'fragment-end': {
        const open = openFragments.pop()
        if (!open) break
        y += FRAGMENT_PADDING
        fragments.push(placeFragment(open, y, scene, centreOf, widths))
        touch(...open.touched)
        y += 8
        break
      }
    }
  })

  const lifelineBottom = y + 24
  for (const [participantId, starts] of openActivations) {
    starts.forEach((start, depth) => activations.push(activationBar(participantId, depth, start, lifelineBottom - 8, centreOf)))
  }

  const participants = new Map<string, PlacedParticipant>()
  for (const participant of scene.participants) {
    const width = widths.get(participant.id) ?? PARTICIPANT_MIN_WIDTH
    const centre = centreOf(participant.id)
    participants.set(participant.id, { id: participant.id, x: centre - width / 2, y: boxTop, width, height: PARTICIPANT_HEIGHT, centre, footY: lifelineBottom })
  }
  const boxes = scene.boxes.map((box) => placeBox(box.label, box.participantIds, participants, lifelineBottom))
  const everything: Box[] = [
    ...participants.values(),
    ...[...participants.values()].map((participant) => ({ ...participant, y: participant.footY })),
    ...notes,
    ...fragments,
    ...boxes,
    ...[...messages.values()].map((message) => ({ x: Math.min(message.fromX, message.toX), y: message.y - MESSAGE_LABEL_ROOM, width: Math.abs(message.toX - message.fromX) + (message.self ? SELF_LOOP_WIDTH + 120 : 0), height: MESSAGE_LABEL_ROOM }))
  ]
  const bounds = union(everything) ?? { x: 0, y: 0, width: 0, height: 0 }
  return { participants, messages, notes, fragments, activations, boxes, lifelineTop, lifelineBottom, bounds }
}

function previousIsMessage(steps: readonly SequenceStep[], index: number): boolean {
  for (let at = index - 1; at >= 0; at -= 1) {
    const type = steps[at]!.type
    if (type === 'message') return true
    if (type !== 'activate' && type !== 'deactivate') return false
  }
  return false
}

/**
 * Columns sit side by side at their own width plus a gap, then are pushed apart wherever a
 * message label, a self-call or a note needs more room than that.
 */
function columnCentres(scene: SequenceScene): Map<string, number> {
  const ids = scene.participants.map((participant) => participant.id)
  const index = new Map(ids.map((id, at) => [id, at]))
  const widths = scene.participants.map((participant) => participantWidth(participant.label))
  const gaps = ids.map((_, at) => (at === 0 ? 0 : (widths[at - 1]! + widths[at]!) / 2 + PARTICIPANT_GAP))

  const need = (from: number, to: number, room: number): void => {
    const [left, right] = from < to ? [from, to] : [to, from]
    if (left === right) return
    const span = gaps.slice(left + 1, right + 1).reduce((sum, gap) => sum + gap, 0)
    if (span < room) gaps[right] = gaps[right]! + (room - span)
  }
  for (const step of scene.steps) {
    if (step.type === 'message') {
      const from = index.get(step.from) ?? 0
      const to = index.get(step.to) ?? 0
      const room = textWidth(step.text, LABEL_FONT_SIZE) + 48
      if (from === to) need(from, from + 1 < ids.length ? from + 1 : from, room + SELF_LOOP_WIDTH)
      else need(from, to, room)
    } else if (step.type === 'note' && step.participantIds.length === 2) {
      need(index.get(step.participantIds[0]!) ?? 0, index.get(step.participantIds[1]!) ?? 0, NOTE_WIDTH / 2)
    }
  }
  const centres = new Map<string, number>()
  let x = widths[0] ? widths[0] / 2 : 0
  ids.forEach((id, at) => {
    if (at > 0) x += gaps[at]!
    centres.set(id, x)
  })
  return centres
}

/** A message leaves its sender's activation bar and arrives on its receiver's, when they have one. */
function placeMessage(
  message: SequenceMessage,
  y: number,
  centreOf: (id: string) => number,
  depthOf: (id: string) => number,
  order: ReadonlyMap<string, number>
): PlacedMessage {
  const from = centreOf(message.from)
  const to = centreOf(message.to)
  const barEdge = (id: string, side: number): number => (depthOf(id) > 0 ? side * (ACTIVATION_WIDTH / 2) + (depthOf(id) - 1) * ACTIVATION_NEST : 0)
  if (message.from === message.to) {
    const start = from + barEdge(message.from, 1)
    return { number: message.number, fromX: start, toX: start, y, self: true, labelAt: { x: start + SELF_LOOP_WIDTH + 8, y: y + SELF_LOOP_HEIGHT / 2 } }
  }
  const rightward = (order.get(message.to) ?? 0) > (order.get(message.from) ?? 0)
  const side = rightward ? 1 : -1
  const fromX = from + barEdge(message.from, side)
  const toX = to + barEdge(message.to, -side)
  return { number: message.number, fromX, toX, y, self: false, labelAt: { x: (fromX + toX) / 2, y: y - 8 } }
}

function activationBar(participantId: string, depth: number, top: number, bottom: number, centreOf: (id: string) => number): PlacedActivation {
  return {
    participantId,
    x: centreOf(participantId) - ACTIVATION_WIDTH / 2 + depth * ACTIVATION_NEST,
    y: top,
    width: ACTIVATION_WIDTH,
    height: Math.max(12, bottom - top)
  }
}

function placeNote(
  index: number,
  placement: 'left' | 'right' | 'over',
  participantIds: readonly string[],
  text: string,
  y: number,
  centreOf: (id: string) => number
): PlacedNote {
  const centres = participantIds.map(centreOf)
  const left = Math.min(...centres)
  const right = Math.max(...centres)
  const width = placement === 'over' && centres.length > 1 ? Math.max(NOTE_WIDTH, right - left + 60) : NOTE_WIDTH
  const lines = wrapText(text, width - 24, NOTE_FONT_SIZE, 6)
  const height = 16 + lines.length * NOTE_LINE
  const x = placement === 'left' ? left - width - 14 : placement === 'right' ? right + 14 : (left + right) / 2 - width / 2
  return { index, x, y, width, height, lines }
}

function placeFragment(
  open: OpenFragment,
  bottom: number,
  scene: SequenceScene,
  centreOf: (id: string) => number,
  widths: ReadonlyMap<string, number>
): PlacedFragment {
  const ids = open.touched.size > 0 ? [...open.touched] : scene.participants.map((participant) => participant.id)
  const inset = (open.depth + 1) * FRAGMENT_INSET
  const lefts = ids.map((id) => centreOf(id) - Math.min(70, (widths.get(id) ?? PARTICIPANT_MIN_WIDTH) / 2 - 10))
  const rights = ids.map((id) => centreOf(id) + Math.min(70, (widths.get(id) ?? PARTICIPANT_MIN_WIDTH) / 2 - 10))
  const left = Math.min(...lefts) - 40 + inset
  const labelRoom = textWidth(open.label, LABEL_FONT_SIZE) + textWidth(open.kind, LABEL_FONT_SIZE) + 64
  const right = Math.max(...rights, left + labelRoom) + 40 - inset
  return { kind: open.kind, label: open.label, depth: open.depth, sections: open.sections, x: left, y: open.top, width: right - left, height: bottom - open.top }
}

function placeBox(label: string | null, participantIds: readonly string[], participants: ReadonlyMap<string, PlacedParticipant>, bottom: number): PlacedBox {
  const members = participantIds.map((id) => participants.get(id)).filter((placed): placed is PlacedParticipant => placed !== undefined)
  const left = Math.min(...members.map((member) => member.x)) - BOX_PADDING
  const right = Math.max(...members.map((member) => member.x + member.width)) + BOX_PADDING
  return { label, x: left, y: 0, width: right - left, height: bottom + PARTICIPANT_HEIGHT + BOX_PADDING }
}
