import { describe, expect, it } from 'vitest'
import { excerptAround, searchDecisions } from '@/features/decisions/lib/decision-search'
import type { Decision } from '@/features/decisions/model/decision'
import { answered, decision, onHold } from './fixtures'

const SPOKEN_AT = '2026-11-23T14:34:00Z'

function found(entry: Decision, query: string) {
  return searchDecisions([entry], query)?.get(entry.id)
}

describe('searching the decisions', () => {
  it('searches nothing until something is typed', () => {
    expect(searchDecisions([decision('a')], '')).toBeNull()
    expect(searchDecisions([decision('a')], '   ')).toBeNull()
  })

  it('keeps the decisions that contain the string as typed, ignoring case', () => {
    const matching = answered('formats', 'PDF and Word')
    const other = answered('size', 'A4')

    const finds = searchDecisions([matching, other], ' pdf AND ')

    expect([...finds!.keys()]).toEqual([matching.id])
    expect(searchDecisions([matching], 'word pdf')!.size).toBe(0)
  })

  it.each([
    ['the topic', decision('a', { topic: 'Zebra' })],
    ['General, for a decision with no topic', decision('a', { question: 'Zebra?', topic: null })],
    ['an offered option of an open question', decision('a', { options: [{ label: 'Zebra', effect: null, recommended: false }] })],
    ['the note of a hold', onHold('a', 'Zebra')],
    ['the replaced answer', answered('a', '3', { previousAnswer: 'Zebra' })],
    ['a later note', answered('a', '3', { details: [{ text: 'Zebra', addedAt: SPOKEN_AT }] })]
  ])('finds %s in the closed row', (_place, entry) => {
    expect(found(entry, entry.topic === null && entry.question === 'Zebra?' ? 'general' : 'zebra')).toEqual({ inRow: true, elsewhere: [] })
  })

  it('names every place outside the row that holds the string, with its first text there', () => {
    const entry = answered('store', 'Disk', {
      why: 'A zebra would ask.',
      options: [{ label: 'Disk', effect: 'No zebra.', recommended: false }],
      lines: [
        { speaker: 'CLAUDE', text: 'First zebra', spokenAt: SPOKEN_AT },
        { speaker: 'PERSON', text: 'Second zebra', spokenAt: SPOKEN_AT }
      ]
    })

    expect(found(entry, 'zebra')).toEqual({
      inRow: false,
      elsewhere: [
        { place: 'Conversation', text: 'First zebra', shownWhenOpened: true },
        { place: 'Options', text: 'No zebra.', shownWhenOpened: true },
        { place: 'Why asked', text: 'A zebra would ask.', shownWhenOpened: true }
      ]
    })
  })

  it('tells apart what an unanswered decision keeps out of its row, and that its notes are not shown opened either', () => {
    const entry = onHold('store', 'Waiting', { previousAnswer: 'zebra', details: [{ text: 'Ask about the zebra', addedAt: SPOKEN_AT }] })

    expect(found(entry, 'zebra')?.elsewhere).toEqual([
      { place: 'Earlier answer', text: 'zebra', shownWhenOpened: true },
      { place: 'Notes', text: 'Ask about the zebra', shownWhenOpened: false }
    ])
  })

  it('does not search the key or who asked', () => {
    expect(found(decision('zebra', { question: 'Which store?' }), 'zebra')).toBeUndefined()
    expect(found(decision('a'), 'asked in the session')).toBeUndefined()
  })
})

describe('the excerpt around a match', () => {
  it('leaves a match near the start with the whole text', () => {
    expect(excerptAround('Do we overwrite it?', 'overwrite')).toBe('Do we overwrite it?')
  })

  it('cuts a long lead at a word, a few words before the match', () => {
    const text = 'If a file with the same name is already there, do we overwrite it or add a number?'

    expect(excerptAround(text, 'OVERWRITE')).toBe('…the same name is already there, do we overwrite it or add a number?')
  })

  it('cuts inside a word when there is no space before the match', () => {
    const text = `${'a'.repeat(80)}zebra`

    expect(excerptAround(text, 'zebra')).toBe(`…${'a'.repeat(40)}zebra`)
  })
})
