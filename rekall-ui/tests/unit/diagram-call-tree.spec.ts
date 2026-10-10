import { describe, expect, it } from 'vitest'
import { callOf, callsInto, callTreeOf, messageNumbersIn, type CallNode } from '@/common/diagram/call-tree'
import type { MessageLine, SequenceScene, SequenceStep } from '@/common/diagram/scene/sequence-scene'

function sequence(...messages: [from: string, to: string, text: string, line?: MessageLine][]): SequenceScene {
  const steps: SequenceStep[] = messages.map(([from, to, text, line], index) => ({
    type: 'message',
    number: index + 1,
    from,
    to,
    text,
    line: line ?? 'solid',
    head: 'arrow'
  }))
  return { participants: [], boxes: [], steps, autonumber: false }
}

function shape(nodes: readonly CallNode[]): unknown[] {
  return nodes.map((node) => [node.call.text, node.reply?.text ?? null, shape(node.children)])
}

const CHECKOUT = sequence(
  ['U', 'S', 'place'],
  ['S', 'V', 'validate'],
  ['V', 'S', 'ok', 'dashed'],
  ['S', 'P', 'charge'],
  ['P', 'B', 'authorise'],
  ['B', 'P', 'approved', 'dashed'],
  ['P', 'S', 'receipt', 'dashed'],
  ['S', 'S', 'store'],
  ['S', 'U', 'done', 'dashed']
)

describe('callTreeOf', () => {
  it('nests each call under the call its sender is answering and closes it with its reply', () => {
    expect(shape(callTreeOf(CHECKOUT))).toEqual([
      ['place', 'done', [['validate', 'ok', []], ['charge', 'receipt', [['authorise', 'approved', []]]], ['store', null, []]]]
    ])
  })

  it('a reply closes the calls opened inside the one it answers', () => {
    const tree = callTreeOf(sequence(['A', 'B', 'one'], ['B', 'C', 'two'], ['B', 'A', 'back', 'dashed'], ['A', 'D', 'three']))
    expect(shape(tree)).toEqual([['one', 'back', [['two', null, []]]], ['three', null, []]])
  })

  it('a reply that answers nothing open is kept as a leaf', () => {
    expect(shape(callTreeOf(sequence(['A', 'B', 'event', 'dashed'])))).toEqual([['event', null, []]])
  })
})

describe('reading the tree', () => {
  const tree = callTreeOf(CHECKOUT)

  it('finds the outermost calls into a participant', () => {
    expect(callsInto(tree, 'P').map((node) => node.call.text)).toEqual(['charge'])
    expect(callsInto(tree, 'S').map((node) => node.call.text)).toEqual(['place'])
  })

  it('finds the call a message opens or answers, and every message under it', () => {
    expect(callOf(tree, 7)?.call.text).toBe('charge')
    expect(messageNumbersIn(callOf(tree, 4)!)).toEqual([4, 7, 5, 6])
  })
})
