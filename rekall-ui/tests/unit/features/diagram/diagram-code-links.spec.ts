import { describe, expect, it } from 'vitest'
import { linkWords } from '@/features/diagram/lib/code-words'
import { filesOfWalkthrough, firstBeatInFile, linkedMethods, methodName } from '@/features/diagram/lib/walkthrough/code-links'
import type { WalkthroughBeat } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import type { CodeLinks } from '@/features/diagram/model/diagram'

function beat(number: number, motion: WalkthroughBeat['motion']): WalkthroughBeat {
  return {
    key: `#${number}`,
    message: { type: 'message', number, from: 'A', to: 'B', text: `m${number}`, line: motion === 'reply' ? 'dashed' : 'solid', head: 'arrow' },
    motion,
    depth: 0,
    explain: null,
    input: null,
    output: null,
    opens: null,
    answers: null
  }
}

const BEATS = [beat(1, 'call'), beat(2, 'call'), beat(3, 'reply'), beat(4, 'call')]

const LINKS: CodeLinks = {
  '#1': [{ file: 'src/controller.rs', startLine: 4, endLine: 9, symbol: 'OrderController::place' }],
  '#2': [{ file: 'src/service.rs', startLine: 10, endLine: 20, symbol: 'OrderService::save' }],
  '#3': [{ file: 'src/service.rs', startLine: 21, endLine: 21, symbol: 'OrderService::save' }],
  '#4': [{ file: 'src/service.rs', startLine: 30, endLine: 40, symbol: 'save(order)' }]
}

describe('the files of a walkthrough', () => {
  it('lists each file once, in the order the walk reaches it', () => {
    expect(filesOfWalkthrough(BEATS, LINKS)).toEqual(['src/controller.rs', 'src/service.rs'])
  })

  it('finds the first step that lives in a file', () => {
    expect(firstBeatInFile(BEATS, LINKS, 'src/service.rs')).toBe(1)
    expect(firstBeatInFile(BEATS, LINKS, 'src/other.rs')).toBe(-1)
  })
})

describe('the method a symbol names', () => {
  it.each([
    ['OrderService::place', 'place'],
    ['Order.place(order)', 'place'],
    ['place', 'place'],
    ['a path/to/file', 'file'],
    ['', null],
    ['()', null]
  ])('reads %s as %s', (symbol, name) => {
    expect(methodName(symbol)).toBe(name)
  })

  it('has no name when there is no symbol', () => {
    expect(methodName(null)).toBeNull()
  })
})

describe('the methods a reader can click', () => {
  it('maps a method to its step and leaves replies out', () => {
    const methods = linkedMethods(BEATS, LINKS, 0)
    expect(methods.get('place')).toBe(0)
    expect(methods.get('save')).toBe(1)
  })

  it('sends a name shared by steps to the next one still to come, else the first', () => {
    expect(linkedMethods(BEATS, LINKS, 2).get('save')).toBe(3)
    expect(linkedMethods(BEATS, LINKS, 4).get('save')).toBe(1)
  })
})

describe('the words of a line that call a method with a step', () => {
  const STEPS = new Map([['insert', 1]])

  it('stands the called word alone and keeps the rest as it was', () => {
    const segments = linkWords([{ text: '    db.insert(order);', classes: '' }], STEPS)
    expect(segments).toEqual([
      { text: '    db.', classes: '', step: null },
      { text: 'insert', classes: '', step: 1 },
      { text: '(order);', classes: '', step: null }
    ])
  })

  it('finds the call even when the parenthesis is in the next token', () => {
    const segments = linkWords([{ text: 'insert', classes: 'hljs-title' }, { text: '(order)', classes: '' }], STEPS)
    expect(segments[0]).toEqual({ text: 'insert', classes: 'hljs-title', step: 1 })
  })

  it('links a method reached through a type but not a function reached through a module', () => {
    const typed = linkWords([{ text: 'Store::insert(order)', classes: '' }], STEPS)
    expect(typed.some((segment) => segment.step === 1)).toBe(true)
    const modular = linkWords([{ text: 'store::insert(order)', classes: '' }], STEPS)
    expect(modular.every((segment) => segment.step === null)).toBe(true)
  })

  it('does not link a word that is not called, or one inside a comment or a string', () => {
    expect(linkWords([{ text: 'let insert = 1;', classes: '' }], STEPS).every((segment) => segment.step === null)).toBe(true)
    expect(linkWords([{ text: '// insert(order)', classes: 'hljs-comment' }], STEPS).every((segment) => segment.step === null)).toBe(true)
    expect(linkWords([{ text: '"insert(order)"', classes: 'hljs-string' }], STEPS).every((segment) => segment.step === null)).toBe(true)
  })
})
