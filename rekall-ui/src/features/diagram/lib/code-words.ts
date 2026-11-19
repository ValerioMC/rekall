import type { CodeToken } from '@/features/diagram/lib/code-highlight'

/** A run of one line's code; `step` is the walkthrough step the word calls, when it does. */
export interface CodeSegment extends CodeToken {
  readonly step: number | null
}

const WORD = /[A-Za-z_]\w*/g
const QUIET_CLASSES = ['hljs-comment', 'hljs-string']

/** Splits a highlighted line so each called method with a step stands alone; a word in a comment or a string never counts. */
export function linkWords(tokens: readonly CodeToken[], steps: ReadonlyMap<string, number>): CodeSegment[] {
  const line = tokens.map((token) => token.text).join('')
  const segments: CodeSegment[] = []
  let offset = 0
  for (const token of tokens) {
    const quiet = QUIET_CLASSES.some((name) => token.classes.split(' ').includes(name))
    let cursor = 0
    if (!quiet) {
      for (const match of token.text.matchAll(WORD)) {
        const step = steps.get(match[0])
        const start = offset + match.index
        if (step === undefined || line[start + match[0].length] !== '(' || isModulePath(line, start)) continue
        if (match.index > cursor) segments.push({ text: token.text.slice(cursor, match.index), classes: token.classes, step: null })
        segments.push({ text: match[0], classes: token.classes, step })
        cursor = match.index + match[0].length
      }
    }
    if (cursor < token.text.length) segments.push({ text: token.text.slice(cursor), classes: token.classes, step: null })
    offset += token.text.length
  }
  return segments
}

/** `fs::read` is a function of a module, not a method of the code the walkthrough follows: a path through a lowercase name never counts. */
function isModulePath(line: string, wordStart: number): boolean {
  if (line.slice(wordStart - 2, wordStart) !== '::') return false
  const qualifier = line.slice(0, wordStart - 2).match(/[A-Za-z_]\w*$/)?.[0]
  return qualifier !== undefined && !/^[A-Z]/.test(qualifier)
}
