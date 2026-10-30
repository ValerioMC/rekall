import { splitLines } from './lines'

export interface DiffLine {
  readonly sign: '+' | '-'
  readonly text: string
}

/** What an edit takes out and puts in, as the event carries no context lines or numbers. */
export function diffLines(oldString: string, newString: string): DiffLine[] {
  return [
    ...splitLines(oldString).map((text): DiffLine => ({ sign: '-', text })),
    ...splitLines(newString).map((text): DiffLine => ({ sign: '+', text }))
  ]
}

export function writeLines(content: string): DiffLine[] {
  return splitLines(content).map((text): DiffLine => ({ sign: '+', text }))
}
