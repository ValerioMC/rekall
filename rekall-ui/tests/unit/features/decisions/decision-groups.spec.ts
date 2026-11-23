import { describe, expect, it } from 'vitest'
import { groupByTopic } from '@/features/decisions/lib/decision-groups'
import { countDecisions, describeTally } from '@/features/decisions/lib/decision-tally'
import { answered, decision, onHold } from './fixtures'

describe('groupByTopic', () => {
  it('puts a topic with an open question first and keeps the others in the order they appear', () => {
    const groups = groupByTopic([
      answered('namespace', 'billing', { topic: 'Cluster' }),
      answered('host', 'billing.internal', { topic: 'Ingress' }),
      decision('db-password', { topic: 'Secrets' })
    ])

    expect(groups.map((group) => group.topic)).toEqual(['Secrets', 'Cluster', 'Ingress'])
  })

  it('lists the open questions of a topic before the rest, each keeping the server order', () => {
    const [group] = groupByTopic([
      answered('a', 'x', { topic: 'Secrets' }),
      decision('b', { topic: 'Secrets' }),
      onHold('c', 'checking', { topic: 'Secrets' }),
      decision('d', { topic: 'Secrets' })
    ])

    expect(group!.decisions.map((entry) => entry.key)).toEqual(['b', 'd', 'a', 'c'])
    expect(group!.answered).toBe(1)
  })

  it('files a decision with no topic under General', () => {
    expect(groupByTopic([answered('a', 'x')]).map((group) => group.topic)).toEqual(['General'])
  })
})

describe('describeTally', () => {
  it('says nothing is recorded when there is nothing', () => {
    expect(describeTally(countDecisions([]))).toEqual({ headline: 'Nothing recorded yet.', rest: '' })
  })

  it('counts the open questions and the ones on hold', () => {
    const tally = describeTally(countDecisions([decision('a'), answered('b', 'x'), onHold('c', 'checking')]))

    expect(tally).toEqual({ headline: '1 question still open.', rest: '1 of 3 answered, 1 on hold. Loaded when it runs.' })
  })

  it('says everything is settled only when nothing is open or on hold', () => {
    expect(describeTally(countDecisions([answered('a', 'x'), answered('b', 'y')])).headline).toBe('Everything is settled.')
    expect(describeTally(countDecisions([answered('a', 'x'), onHold('b', 'checking')])).headline).toBe('Nothing is open.')
  })
})
