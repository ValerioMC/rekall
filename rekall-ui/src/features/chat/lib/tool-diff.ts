import { splitLines } from './lines'
import type { FileChange, ToolInput } from '../model/chat-event'

/** `' '` is an unchanged line of a unified diff and `'@'` the header of one of its hunks. */
export interface DiffLine {
  readonly sign: '+' | '-' | ' ' | '@'
  readonly text: string
}

export interface FileDiff {
  readonly path: string
  readonly note: string | null
  readonly lines: readonly DiffLine[]
}

/** The files a call changes, each with its lines; a call that changes none has none. */
export function fileDiffs(input: ToolInput): FileDiff[] {
  switch (input.tool) {
    case 'edit':
      return [
        {
          path: input.filePath,
          note: input.replaceAll ? 'all occurrences' : null,
          lines: diffLines(input.oldString, input.newString)
        }
      ]
    case 'write':
      return [{ path: input.filePath, note: null, lines: writeLines(input.content) }]
    case 'fileChanges':
      return input.changes.map(changeDiff)
    default:
      return []
  }
}

/** What an edit takes out and puts in, as the event carries no context lines or numbers. */
export function diffLines(oldString: string, newString: string): DiffLine[] {
  return [...removedLines(oldString), ...writeLines(newString)]
}

export function writeLines(content: string): DiffLine[] {
  return splitLines(content).map((text): DiffLine => ({ sign: '+', text }))
}

/** The hunks of a unified diff; whatever precedes the first one or belongs to none is left out. */
export function unifiedLines(diff: string): DiffLine[] {
  const lines = splitLines(diff)
  const firstHunk = lines.findIndex((line) => line.startsWith('@@'))
  if (firstHunk < 0) return []
  return lines.slice(firstHunk).flatMap(unifiedLine)
}

function unifiedLine(line: string): DiffLine[] {
  if (line.startsWith('@@')) return [{ sign: '@', text: line }]
  const sign = line.charAt(0)
  return sign === '+' || sign === '-' || sign === ' ' ? [{ sign, text: line.slice(1) }] : []
}

function removedLines(content: string): DiffLine[] {
  return splitLines(content).map((text): DiffLine => ({ sign: '-', text }))
}

function changeDiff(change: FileChange): FileDiff {
  return {
    path: change.path,
    note: change.movedTo ? `moved to ${change.movedTo}` : null,
    lines: changeLines(change)
  }
}

function changeLines(change: FileChange): DiffLine[] {
  switch (change.kind) {
    case 'add':
      return writeLines(change.diff)
    case 'delete':
      return removedLines(change.diff)
    case 'update':
      return unifiedLines(change.diff)
  }
}
