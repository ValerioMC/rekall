import { z } from 'zod'
import { asRecord, requireGetters } from './mermaid-db'
import { plainLabel } from './mermaid-text'
import type {
  FragmentKind,
  MessageHead,
  MessageLine,
  SequenceBox,
  SequenceParticipant,
  SequenceScene,
  SequenceStep
} from '../scene/sequence-scene'

/**
 * Mermaid's sequence database read into a `SequenceScene`. The numbers are Mermaid's
 * `LINETYPE` codes; a code this reader does not know is skipped rather than guessed.
 */

const ActorSchema = z.object({
  name: z.string(),
  description: z.string().optional(),
  type: z.string().optional()
})

const BoxSchema = z.object({
  name: z.string().nullish(),
  actorKeys: z.array(z.string()).default([])
})

const StepSchema = z.object({
  type: z.number().optional(),
  from: z.string().optional(),
  to: z.string().optional(),
  message: z.unknown(),
  placement: z.number().optional()
})

type RawStep = z.infer<typeof StepSchema>

const SOLID_LINES: ReadonlySet<number> = new Set([0, 3, 5, 24, 33, 41, 42, 43, 44, 45, 46, 47, 48, 59, 60, 61])
const DASHED_LINES: ReadonlySet<number> = new Set([1, 4, 6, 25, 34, 51, 52, 53, 54, 55, 56, 57, 58])

const HEADS: Readonly<Record<number, MessageHead>> = {
  3: 'cross',
  4: 'cross',
  5: 'none',
  6: 'none',
  24: 'async',
  25: 'async',
  33: 'both',
  34: 'both'
}

const FRAGMENT_STARTS: Readonly<Record<number, FragmentKind>> = {
  10: 'loop',
  12: 'alt',
  15: 'opt',
  19: 'par',
  22: 'rect',
  27: 'critical',
  30: 'break',
  32: 'par'
}

const FRAGMENT_SECTIONS: ReadonlySet<number> = new Set([13, 20, 28])
const FRAGMENT_ENDS: ReadonlySet<number> = new Set([11, 14, 16, 21, 23, 29, 31])
const NOTE = 2
const ACTIVATE = 17
const DEACTIVATE = 18
const AUTONUMBER = 26
const PLACEMENTS = ['left', 'right', 'over'] as const

export function readSequence(db: unknown): SequenceScene {
  const source = requireGetters(db, ['getActors', 'getMessages', 'getBoxes'])
  const actors = z.record(ActorSchema).parse(asRecord(source.getActors()))
  const participants: SequenceParticipant[] = Object.values(actors).map((actor) => ({
    id: actor.name,
    label: plainLabel(actor.description) || actor.name,
    isActor: actor.type === 'actor'
  }))
  const boxes: SequenceBox[] = z
    .array(BoxSchema)
    .parse(source.getBoxes())
    .map((box) => ({ label: plainLabel(box.name) || null, participantIds: box.actorKeys }))
  const raw = z.array(StepSchema).parse(source.getMessages())
  return {
    participants,
    boxes,
    steps: stepsOf(raw),
    autonumber: raw.some((step) => step.type === AUTONUMBER)
  }
}

function stepsOf(raw: readonly RawStep[]): SequenceStep[] {
  const steps: SequenceStep[] = []
  let number = 0
  for (const step of raw) {
    const code = step.type ?? -1
    const text = typeof step.message === 'string' ? plainLabel(step.message) : ''
    if ((SOLID_LINES.has(code) || DASHED_LINES.has(code)) && step.from && step.to) {
      number += 1
      const line: MessageLine = DASHED_LINES.has(code) ? 'dashed' : 'solid'
      steps.push({ type: 'message', number, from: step.from, to: step.to, text, line, head: HEADS[code] ?? 'arrow' })
    } else if (code === NOTE && step.from) {
      const participantIds = step.to && step.to !== step.from ? [step.from, step.to] : [step.from]
      steps.push({ type: 'note', placement: PLACEMENTS[step.placement ?? 2] ?? 'over', participantIds, text })
    } else if (FRAGMENT_STARTS[code]) {
      steps.push({ type: 'fragment-start', kind: FRAGMENT_STARTS[code], label: FRAGMENT_STARTS[code] === 'rect' ? '' : text })
    } else if (FRAGMENT_SECTIONS.has(code)) {
      steps.push({ type: 'fragment-section', label: text })
    } else if (FRAGMENT_ENDS.has(code)) {
      steps.push({ type: 'fragment-end' })
    } else if ((code === ACTIVATE || code === DEACTIVATE) && step.from) {
      steps.push({ type: code === ACTIVATE ? 'activate' : 'deactivate', participantId: step.from })
    }
  }
  return steps
}
