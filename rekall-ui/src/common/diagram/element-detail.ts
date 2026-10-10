import { MESSAGE_STYLE, roleStyle, type ElementStyle } from './element-style'
import { messagesOf, type SequenceScene } from './scene/sequence-scene'
import type { GraphScene } from './scene/graph-scene'

/**
 * What the element panel says about the element the reader picked: what it is called, what
 * kind of thing it is, and the key its code links are stored under.
 */
export interface ElementDetail {
  readonly key: string
  readonly linkKey: string
  readonly title: string
  readonly subtitle: string | null
  readonly look: ElementStyle
}

export function sequenceElement(scene: SequenceScene, key: string): ElementDetail | null {
  const labels = new Map(scene.participants.map((participant) => [participant.id, participant.label]))
  if (key.startsWith('#')) {
    const message = messagesOf(scene).find((candidate) => `#${candidate.number}` === key)
    if (!message) return null
    const verb = message.line === 'dashed' ? 'Reply' : 'Call'
    const look = { ...MESSAGE_STYLE, label: `${verb} ${key}` }
    return { key, linkKey: key, title: message.text || '(no text)', subtitle: `${labels.get(message.from) ?? message.from} → ${labels.get(message.to) ?? message.to}`, look }
  }
  const participant = scene.participants.find((candidate) => candidate.id === key)
  if (!participant) return null
  const subtitle = participant.label === participant.id ? null : participant.id
  return { key, linkKey: key, title: participant.label, subtitle, look: roleStyle(participant.isActor ? 'actor' : 'participant') }
}

export function graphElement(scene: GraphScene, key: string): ElementDetail | null {
  const node = scene.nodes.find((candidate) => candidate.id === key)
  if (node) {
    const subtitle = node.linkKey === node.label ? null : node.linkKey
    return { key, linkKey: node.linkKey, title: node.label || node.id, subtitle, look: roleStyle(node.role) }
  }
  const group = scene.groups.find((candidate) => candidate.id === key)
  if (!group) return null
  return { key, linkKey: group.id, title: group.label || group.id, subtitle: null, look: { ...roleStyle('state'), label: 'Group' } }
}

/** A line to or from the picked element, as the panel lists it: a way to walk on. */
export interface Connection {
  readonly edgeKey: string
  readonly otherKey: string
  readonly otherTitle: string
  readonly label: string | null
  readonly look: ElementStyle
}

export function connectionsOf(scene: GraphScene, key: string): { readonly outgoing: Connection[]; readonly incoming: Connection[] } {
  const titled = (id: string): { title: string; look: ElementStyle } => {
    const node = scene.nodes.find((candidate) => candidate.id === id)
    if (node) return { title: node.label || (node.role === 'pseudo' ? (node.outline === 'start' ? 'start' : 'end') : node.id), look: roleStyle(node.role) }
    return { title: scene.groups.find((group) => group.id === id)?.label ?? id, look: roleStyle('state') }
  }
  const connection = (edgeKey: string, otherKey: string, label: string | null): Connection => ({ edgeKey, otherKey, label, otherTitle: titled(otherKey).title, look: titled(otherKey).look })
  return {
    outgoing: scene.edges.filter((edge) => edge.from === key).map((edge) => connection(edge.key, edge.to, edge.label)),
    incoming: scene.edges.filter((edge) => edge.to === key).map((edge) => connection(edge.key, edge.from, edge.label))
  }
}
