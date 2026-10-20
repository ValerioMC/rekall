import { callOf, type CallNode } from '../call-tree'
import { messageKey, messagesOf, type SequenceMessage, type SequenceScene } from '../scene/sequence-scene'
import type { Spotlight } from '../emphasis'
import type { Walkthrough } from '@/features/diagram/model/diagram'

/**
 * A sequence played one message at a time. Each beat is a message in order, with how it moves,
 * how deep in the call tree it sits, and what the walkthrough says about it. A reply the
 * walkthrough says nothing about still shows what its call returns.
 */

export type BeatMotion = 'call' | 'reply' | 'self'

export interface WalkthroughBeat {
  readonly key: string
  readonly message: SequenceMessage
  readonly motion: BeatMotion
  readonly depth: number
  readonly explain: string | null
  readonly input: string | null
  readonly output: string | null
  readonly opens: number | null
  /** For a reply, the number of the call it answers. */
  readonly answers: number | null
}

export function walkthroughBeats(scene: SequenceScene, calls: readonly CallNode[], walkthrough: Walkthrough | null): WalkthroughBeat[] {
  const depths = depthsOf(calls)
  const words = walkthrough?.messages ?? {}
  return messagesOf(scene).map((message) => {
    const key = messageKey(message.number)
    const own = words[key]
    const motion = motionOf(message)
    const answers = motion === 'reply' ? answeredCall(calls, message) : null
    const call = answers === null ? undefined : words[messageKey(answers)]
    return {
      key,
      message,
      motion,
      depth: depths.get(message.number) ?? 0,
      explain: own?.explain ?? null,
      input: own?.input ?? null,
      output: own?.output ?? call?.output ?? null,
      opens: own?.opens ?? null,
      answers
    }
  })
}

/** The first beat that reaches a participant: where a click on it starts its part of the story. */
export function firstBeatInto(beats: readonly WalkthroughBeat[], participantId: string): number {
  const reaching = beats.findIndex((beat) => beat.message.to === participantId)
  return reaching >= 0 ? reaching : beats.findIndex((beat) => beat.message.from === participantId)
}

/** What has happened up to `position` stays lit; what is still to come waits dimmed. */
export function walkthroughSpotlight(beats: readonly WalkthroughBeat[], position: number): Spotlight | null {
  const current = beats[position]
  if (!current) return null
  const played = beats.slice(0, position + 1)
  return {
    selected: current.key,
    nodes: new Set(played.flatMap((beat) => [beat.message.from, beat.message.to])),
    edges: new Set(played.map((beat) => beat.key))
  }
}

function motionOf(message: SequenceMessage): BeatMotion {
  if (message.from === message.to) return 'self'
  return message.line === 'dashed' ? 'reply' : 'call'
}

function answeredCall(calls: readonly CallNode[], reply: SequenceMessage): number | null {
  const node = callOf(calls, reply.number)
  return node && node.reply?.number === reply.number ? node.call.number : null
}

/** A call and its reply sit at the depth of the call; what it calls sits one deeper. */
function depthsOf(nodes: readonly CallNode[], depth = 0, into = new Map<number, number>()): Map<number, number> {
  for (const node of nodes) {
    into.set(node.call.number, depth)
    if (node.reply) into.set(node.reply.number, depth)
    depthsOf(node.children, depth + 1, into)
  }
  return into
}
