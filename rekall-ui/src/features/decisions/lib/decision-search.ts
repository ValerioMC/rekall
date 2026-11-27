import { matchParts } from '@/common/format/match-parts'
import { GENERAL_TOPIC } from '@/features/decisions/lib/decision-groups'
import { isAnswered, isOnHold, isOpen, type Decision } from '@/features/decisions/model/decision'
import type { DecisionId } from '@/model/branded'

export type DecisionPlace = 'Conversation' | 'Options' | 'Why asked' | 'Earlier answer' | 'Notes'

export interface PlaceFound {
  readonly place: DecisionPlace
  readonly text: string
  readonly shownWhenOpened: boolean
}

/** Where a searched string was found in one decision: in what its closed row shows, and in the places it does not. */
export interface DecisionFind {
  readonly inRow: boolean
  readonly elsewhere: readonly PlaceFound[]
}

export type DecisionFinds = ReadonlyMap<DecisionId, DecisionFind>

interface SearchedPlace {
  readonly place: DecisionPlace
  readonly texts: readonly (string | null)[]
  readonly shownWhenOpened: boolean
}

const EXCERPT_LEAD = 40
const EXCERPT_UNCUT = 48

function contains(text: string | null, query: string): text is string {
  return text !== null && matchParts(text, query).some((part) => part.hit)
}

function closedRowTexts(decision: Decision): readonly (string | null)[] {
  const heading = [decision.topic ?? GENERAL_TOPIC, decision.question]
  if (isOpen(decision)) return [...heading, ...decision.options.map((option) => option.label)]
  if (isOnHold(decision)) return [...heading, decision.holdNote]
  return [...heading, decision.answer, decision.previousAnswer, ...decision.details.map((detail) => detail.text)]
}

/** The text a closed row leaves out. Notes of an unanswered decision are not shown by the opened row either. */
function placesOutsideRow(decision: Decision): readonly SearchedPlace[] {
  const options = decision.options.flatMap((option) => (isOpen(decision) ? [option.effect] : [option.label, option.effect]))
  const places: SearchedPlace[] = [
    { place: 'Conversation', texts: decision.lines.map((line) => line.text), shownWhenOpened: true },
    { place: 'Options', texts: options, shownWhenOpened: true },
    { place: 'Why asked', texts: [decision.why], shownWhenOpened: true }
  ]
  if (isAnswered(decision)) return places
  return [
    ...places,
    { place: 'Earlier answer', texts: [decision.previousAnswer], shownWhenOpened: true },
    { place: 'Notes', texts: decision.details.map((detail) => detail.text), shownWhenOpened: false }
  ]
}

function findIn(decision: Decision, query: string): DecisionFind | undefined {
  const inRow = closedRowTexts(decision).some((text) => contains(text, query))
  const elsewhere = placesOutsideRow(decision).flatMap(({ place, texts, shownWhenOpened }) => {
    const text = texts.find((candidate) => contains(candidate, query))
    return typeof text === 'string' ? [{ place, text, shownWhenOpened }] : []
  })
  return inRow || elsewhere.length > 0 ? { inRow, elsewhere } : undefined
}

/** The decisions that contain the string, ignoring case, or `null` when there is nothing to search for. */
export function searchDecisions(decisions: readonly Decision[], query: string): DecisionFinds | null {
  if (query.trim() === '') return null
  const finds = new Map<DecisionId, DecisionFind>()
  for (const decision of decisions) {
    const find = findIn(decision, query)
    if (find !== undefined) finds.set(decision.id, find)
  }
  return finds
}

/** A long text cut from a few words before the first match, so a single line still shows what was found. */
export function excerptAround(text: string, query: string): string {
  const at = text.toLowerCase().indexOf(query.trim().toLowerCase())
  if (at <= EXCERPT_UNCUT) return text
  const space = text.indexOf(' ', at - EXCERPT_LEAD)
  const from = space >= 0 && space < at ? space + 1 : at - EXCERPT_LEAD
  return `…${text.slice(from)}`
}
