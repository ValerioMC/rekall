import { describe, expect, it } from 'vitest'
import { answeredCount, groupByPass, passDetail } from '@/features/feedback/lib/feedback-passes'
import { applied, appliedInPass, entry, running } from './fixtures'

describe('groupByPass', () => {
  it('puts the waiting entries in the next run, whatever their kind', () => {
    const groups = groupByPass([entry(1), entry(2, { kind: 'PLAN' }), entry(3, { kind: 'REVISION', quotedMarkdown: 'a' })])

    expect(groups.next.map((feedback) => feedback.seq)).toEqual([1, 2, 3])
    expect(groups.passes).toEqual([])
  })

  it('groups the entries of one pass together, newest pass first, with the running one marked', () => {
    const groups = groupByPass([appliedInPass(1, 1, 'Done.'), appliedInPass(2, 1, 'Done too.'), running(3, 2), running(4, 2), entry(5)])

    expect(groups.passes.map((pass) => [pass.pass, pass.running, pass.entries.map((feedback) => feedback.seq)])).toEqual([
      [2, true, [3, 4]],
      [1, false, [1, 2]]
    ])
    expect(groups.next.map((feedback) => feedback.seq)).toEqual([5])
  })

  it('says when a pass was taken and, once every entry is answered, when it was finished', () => {
    const groups = groupByPass([
      appliedInPass(1, 1, 'Done.', { takenAt: '2026-11-04T10:00:00Z', appliedAt: '2026-11-04T10:20:00Z' }),
      appliedInPass(2, 1, 'Done.', { takenAt: '2026-11-04T10:00:00Z', appliedAt: '2026-11-04T10:40:00Z' }),
      running(3, 2, { takenAt: '2026-11-04T11:00:00Z' })
    ])

    expect(groups.passes[1]).toMatchObject({ pass: 1, takenAt: '2026-11-04T10:00:00Z', finishedAt: '2026-11-04T10:40:00Z' })
    expect(groups.passes[0]).toMatchObject({ pass: 2, running: true, finishedAt: null })
  })

  it('keeps an entry applied before passes existed apart from every pass', () => {
    const groups = groupByPass([applied(1, 'Old answer.'), appliedInPass(2, 1, 'Done.')])

    expect(groups.before.map((feedback) => feedback.seq)).toEqual([1])
    expect(groups.passes.map((pass) => pass.pass)).toEqual([1])
  })

  it('has nothing for an empty thread', () => {
    expect(groupByPass([])).toEqual({ next: [], passes: [], before: [] })
  })
})

describe('passDetail and answeredCount', () => {
  const NOW = Date.parse('2026-11-04T10:34:00Z')

  it('says when a running pass started and when a done one finished', () => {
    const groups = groupByPass([appliedInPass(1, 1, 'Done.', { appliedAt: '2026-11-04T10:00:00Z' }), running(2, 2, { takenAt: '2026-11-04T10:30:00Z' })])

    expect(passDetail(groups.passes[0]!, NOW)).toBe('started 4 min ago')
    expect(passDetail(groups.passes[1]!, NOW)).toBe('finished 34 min ago')
  })

  it('counts the entries of a pass a session already answered', () => {
    const groups = groupByPass([appliedInPass(1, 1, 'Done.'), running(2, 1), running(3, 1)])

    expect(answeredCount(groups.passes[0]!)).toBe(1)
  })
})
