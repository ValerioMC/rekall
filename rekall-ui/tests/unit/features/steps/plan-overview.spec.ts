import { describe, expect, it } from 'vitest'
import { groupSteps, overviewTitle, queueNote } from '@/features/steps/lib/plan-overview'
import { step } from '../adherence/fixtures'
import type { TaskStep } from '@/features/steps/model/task-step'

describe('grouping the steps by what they ask of the reader', () => {
  it('splits the checklist into what waits, what runs, what is next and what is done, in order', () => {
    const groups = groupSteps([step(1, 'DONE'), step(2, 'CLAIMED'), step(3, 'RUNNING'), step(4, 'OPEN'), step(5, 'CLAIMED')])

    expect(groups.waiting.map((entry) => entry.title)).toEqual(['Step 2', 'Step 5'])
    expect(groups.working.map((entry) => entry.title)).toEqual(['Step 3'])
    expect(groups.upNext.map((entry) => entry.title)).toEqual(['Step 4'])
    expect(groups.done.map((entry) => entry.title)).toEqual(['Step 1'])
  })
})

describe('the overview title', () => {
  it('leads with what waits for the reader, in the singular or the plural', () => {
    expect(overviewTitle(groupSteps([step(1, 'CLAIMED')]), null)).toBe('1 step waits for you')
    expect(overviewTitle(groupSteps([step(1, 'CLAIMED'), step(2, 'CLAIMED')]), null)).toBe('2 steps wait for you')
  })

  it('names the step a session is working on when nothing waits', () => {
    expect(overviewTitle(groupSteps([step(1, 'DONE'), step(2, 'RUNNING')]), 2)).toBe('A session is working on step 2')
  })

  it('says every step is done, or that nothing waits while steps remain', () => {
    expect(overviewTitle(groupSteps([step(1, 'DONE')]), null)).toBe('Every step is done')
    expect(overviewTitle(groupSteps([step(1, 'DONE'), step(2, 'OPEN')]), null)).toBe('Nothing waits for you')
  })
})

describe('the note a step carries in the overview', () => {
  const claimed = { ...step(1, 'CLAIMED', 1), claimedAt: new Date(Date.now() - 14 * 60_000).toISOString() } as TaskStep

  it('sums a claim up from what was found on it', () => {
    expect(queueNote(claimed, { checks: '4 checks passed', commits: 2 })).toBe('4 checks passed · 2 commits · pass 2 · claimed 14 min ago')
  })

  it('leaves out what a claim has not got', () => {
    expect(queueNote(claimed, { checks: null, commits: 0 })).toBe('pass 2 · claimed 14 min ago')
  })

  it('says a session is on a running step, with its commits so far', () => {
    expect(queueNote(step(1, 'RUNNING'), { checks: null, commits: 1 })).toBe('1 commit so far · a session is on it')
    expect(queueNote(step(1, 'RUNNING'), { checks: null, commits: 0 })).toBe('a session is on it')
  })

  it('shows the first line of what an open step says, or that nothing is written', () => {
    const written = { ...step(1, 'OPEN'), bodyMarkdown: '\n  Declare the suite.\nMore' } as TaskStep

    expect(queueNote(written, { checks: null, commits: 0 })).toBe('Declare the suite.')
    expect(queueNote({ ...written, bodyMarkdown: null } as TaskStep, { checks: null, commits: 0 })).toBe('No detail written yet')
  })

  it('says when a step was accepted', () => {
    const done = { ...step(1, 'DONE'), doneAt: new Date(Date.now() - 2 * 3_600_000).toISOString() } as TaskStep

    expect(queueNote(done, { checks: null, commits: 0 })).toBe('Done 2 hours ago')
  })
})
