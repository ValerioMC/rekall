import { describe, expect, it } from 'vitest'
import { wordDiff } from '@/features/commits/lib/word-diff'

describe('wordDiff', () => {
  it('marks only the words an edit changed in both lines', () => {
    const diff = wordDiff('expect(find("terminal-pane"))', 'expect(find("terminal-float"))')

    expect(diff?.removed).toEqual([
      { text: 'expect(find("terminal-', changed: false },
      { text: 'pane', changed: true },
      { text: '"))', changed: false }
    ])
    expect(diff?.added.find((part) => part.changed)?.text).toBe('float')
  })

  it('leaves two unrelated lines unmarked', () => {
    expect(wordDiff('router.push({ name: "terminal" })', 'if (kind === "task") {')).toBeNull()
  })

  it('marks an insertion at the end of a line', () => {
    const diff = wordDiff("pick: [session: Summary]", "pick: [session: Summary, kind: 'task']")

    expect(diff?.removed.some((part) => part.changed)).toBe(false)
    expect(diff?.added.find((part) => part.changed)?.text).toBe(", kind: 'task'")
  })

  it('does not pair a line with an empty one', () => {
    expect(wordDiff('', 'new')).toBeNull()
  })
})
