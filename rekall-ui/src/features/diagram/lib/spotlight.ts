import { callOf, callsInto, messageNumbersIn, type CallNode } from './call-tree'
import { messageKey, messagesOf, type SequenceScene } from './scene/sequence-scene'
import type { GraphScene } from './scene/graph-scene'
import type { Spotlight } from './emphasis'

/**
 * What the canvas lights when the reader picks an element or searches. In a sequence a
 * participant lights every message it sends or receives, and a message lights the calls made
 * under it; in a graph a node lights its neighbours. Element keys are link keys: an id, or `#n`.
 */

const MIN_QUERY = 2

export function sequenceSpotlight(scene: SequenceScene, tree: readonly CallNode[], selected: string | null, query: string): Spotlight | null {
  if (selected) return selected.startsWith('#') ? aroundMessage(scene, tree, selected) : aroundParticipant(scene, selected)
  const needle = query.trim().toLowerCase()
  if (needle.length < MIN_QUERY) return null
  const participants = scene.participants.filter((participant) => matches(needle, participant.id, participant.label)).map((participant) => participant.id)
  const messages = messagesOf(scene).filter((message) => matches(needle, message.text)).map((message) => messageKey(message.number))
  return { selected: null, nodes: new Set(participants), edges: new Set(messages) }
}

function aroundParticipant(scene: SequenceScene, id: string): Spotlight {
  const touching = messagesOf(scene).filter((message) => message.from === id || message.to === id)
  const others = touching.flatMap((message) => [message.from, message.to])
  return { selected: id, nodes: new Set([id, ...others]), edges: new Set(touching.map((message) => messageKey(message.number))) }
}

function aroundMessage(scene: SequenceScene, tree: readonly CallNode[], key: string): Spotlight {
  const number = Number(key.slice(1))
  const call = callOf(tree, number)
  const numbers = call ? messageNumbersIn(call) : [number]
  const lit = messagesOf(scene).filter((message) => numbers.includes(message.number))
  return {
    selected: key,
    nodes: new Set(lit.flatMap((message) => [message.from, message.to])),
    edges: new Set(lit.map((message) => messageKey(message.number)))
  }
}

export function graphSpotlight(scene: GraphScene, selected: string | null, query: string): Spotlight | null {
  if (selected) {
    const touching = scene.edges.filter((edge) => edge.from === selected || edge.to === selected)
    return { selected, nodes: new Set([selected, ...touching.flatMap((edge) => [edge.from, edge.to])]), edges: new Set(touching.map((edge) => edge.key)) }
  }
  const needle = query.trim().toLowerCase()
  if (needle.length < MIN_QUERY) return null
  const ids = scene.nodes.filter((node) => matches(needle, node.id, node.label)).map((node) => node.id)
  return { selected: null, nodes: new Set(ids), edges: new Set() }
}

/** The calls the panel shows for a selection: into a participant, or under a message. */
export function callsFor(tree: readonly CallNode[], selected: string): CallNode[] {
  if (!selected.startsWith('#')) return callsInto(tree, selected)
  const call = callOf(tree, Number(selected.slice(1)))
  return call ? [call] : []
}

function matches(needle: string, ...texts: string[]): boolean {
  return texts.some((text) => text.toLowerCase().includes(needle))
}
