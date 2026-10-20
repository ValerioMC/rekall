import { describe, expect, it } from 'vitest'
import { callTreeOf } from '@/features/diagram/lib/call-tree'
import { firstBeatInto, walkthroughBeats, walkthroughSpotlight } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import { advance, enter, leave, leaveTo, moveTo, retreat, startAt, topOf } from '@/features/diagram/lib/walkthrough/walkthrough-cursor'
import type { MessageLine, SequenceScene, SequenceStep } from '@/features/diagram/lib/scene/sequence-scene'
import type { Walkthrough } from '@/features/diagram/model/diagram'

function sequence(...messages: [from: string, to: string, text: string, line?: MessageLine][]): SequenceScene {
  const steps: SequenceStep[] = messages.map(([from, to, text, line], index) => ({ type: 'message', number: index + 1, from, to, text, line: line ?? 'solid', head: 'arrow' }))
  return { participants: [], boxes: [], steps, autonumber: false }
}

const ORDER = sequence(['C', 'S', 'place(order)'], ['S', 'S', 'validate'], ['S', 'DB', 'insert'], ['DB', 'S', 'id', 'dashed'], ['S', 'C', 'Order', 'dashed'])

const WORDS: Walkthrough = {
  external: ['DB'],
  messages: {
    '#1': { explain: 'Hands the order on', input: '{ sku: "A-7" }', output: 'Order', opens: 1 },
    '#3': { explain: null, input: 'order', output: 'id', opens: null }
  }
}

describe('walkthroughBeats', () => {
  const beats = walkthroughBeats(ORDER, callTreeOf(ORDER), WORDS)

  it('plays every message in order, telling calls, self-calls and replies apart', () => {
    expect(beats.map((beat) => [beat.key, beat.motion, beat.depth])).toEqual([
      ['#1', 'call', 0],
      ['#2', 'self', 1],
      ['#3', 'call', 1],
      ['#4', 'reply', 1],
      ['#5', 'reply', 0]
    ])
  })

  it('carries what the walkthrough says, and lets a silent reply show what its call returns', () => {
    expect(beats[0]).toMatchObject({ explain: 'Hands the order on', input: '{ sku: "A-7" }', opens: 1 })
    expect(beats[3]).toMatchObject({ answers: 3, output: 'id', explain: null })
    expect(beats[4]).toMatchObject({ answers: 1, output: 'Order' })
  })

  it('still plays a sequence that has no walkthrough, with nothing to say', () => {
    const bare = walkthroughBeats(ORDER, callTreeOf(ORDER), null)
    expect(bare).toHaveLength(5)
    expect(bare.every((beat) => beat.explain === null && beat.opens === null)).toBe(true)
  })

  it('starts a participant at the first beat that reaches it', () => {
    expect(firstBeatInto(beats, 'DB')).toBe(2)
    expect(firstBeatInto(beats, 'C')).toBe(4)
    expect(firstBeatInto(beats, 'nobody')).toBe(-1)
  })

  it('lights what has been played and selects the current beat', () => {
    const lit = walkthroughSpotlight(beats, 2)
    expect(lit?.selected).toBe('#3')
    expect([...(lit?.edges ?? [])]).toEqual(['#1', '#2', '#3'])
    expect(lit?.nodes.has('DB')).toBe(true)
    expect(walkthroughSpotlight(beats, 9)).toBeNull()
  })
})

describe('walkthrough cursor', () => {
  it('moves within a sheet and stops at both ends', () => {
    const start = startAt(0)
    expect(retreat(start)).toBeNull()
    const second = advance(start, 3)!
    expect(topOf(second).position).toBe(1)
    expect(advance(moveTo(start, 2, 3), 3)).toBeNull()
    expect(topOf(moveTo(start, 99, 3)).position).toBe(2)
  })

  it('steps into a call and comes back onward at its end, or onto the call when walking back', () => {
    const atCall = moveTo(startAt(0), 1, 5)
    const inside = enter(atCall, 2, 'place(order)')
    expect(topOf(inside)).toEqual({ sheetIndex: 2, position: 0, via: 'place(order)' })
    expect(topOf(advance(moveTo(inside, 3, 4), 4)!)).toEqual({ sheetIndex: 0, position: 2, via: null })
    expect(topOf(retreat(inside)!)).toEqual({ sheetIndex: 0, position: 1, via: null })
    expect(leave(startAt(0), true)).toEqual(startAt(0))
  })

  it('backs out to any frame it went through', () => {
    const deep = enter(enter(startAt(0), 1, 'a'), 2, 'b')
    expect(leaveTo(deep, 0)).toEqual(startAt(0))
    expect(leaveTo(deep, 1).map((frame) => frame.sheetIndex)).toEqual([0, 1])
  })
})
