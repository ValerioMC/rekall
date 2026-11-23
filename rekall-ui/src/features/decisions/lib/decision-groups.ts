import { isAnswered, isOpen, type Decision } from '@/features/decisions/model/decision'

export const GENERAL_TOPIC = 'General'

export interface DecisionGroup {
  readonly topic: string
  readonly decisions: readonly Decision[]
  readonly answered: number
}

/**
 * The decisions under their topics: a topic with a question still open comes first, then the
 * topics in the order they first appear; inside a topic the open questions come before the rest,
 * each keeping the order the server gave. A decision with no topic is filed under General.
 */
export function groupByTopic(decisions: readonly Decision[]): DecisionGroup[] {
  const topics = [...new Set(decisions.map((decision) => decision.topic ?? GENERAL_TOPIC))]
  const waiting = (topic: string): boolean => decisions.some((decision) => (decision.topic ?? GENERAL_TOPIC) === topic && isOpen(decision))
  const ordered = [...topics].sort((a, b) => Number(waiting(b)) - Number(waiting(a)) || topics.indexOf(a) - topics.indexOf(b))
  return ordered.map((topic) => {
    const inTopic = decisions.filter((decision) => (decision.topic ?? GENERAL_TOPIC) === topic)
    const sorted = [...inTopic.filter(isOpen), ...inTopic.filter((decision) => !isOpen(decision))]
    return { topic, decisions: sorted, answered: inTopic.filter(isAnswered).length }
  })
}
