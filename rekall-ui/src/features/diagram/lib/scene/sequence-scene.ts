/**
 * A sequence sheet as the console draws it: who takes part, and what happens in order, read out
 * of Mermaid and free of its rendering. Layout, drawing and the call tree work on this.
 */

export interface SequenceParticipant {
  readonly id: string
  readonly label: string
  readonly isActor: boolean
}

/** A `box` around neighbouring participants. */
export interface SequenceBox {
  readonly label: string | null
  readonly participantIds: readonly string[]
}

/** Solid is a call; dashed is a reply. */
export type MessageLine = 'solid' | 'dashed'

export type MessageHead = 'arrow' | 'none' | 'cross' | 'async' | 'both'

export interface SequenceMessage {
  readonly type: 'message'
  /** One-based, in order: what `autonumber` prints and what a `#n` link addresses. */
  readonly number: number
  readonly from: string
  readonly to: string
  readonly text: string
  readonly line: MessageLine
  readonly head: MessageHead
}

export interface SequenceNote {
  readonly type: 'note'
  readonly placement: 'left' | 'right' | 'over'
  readonly participantIds: readonly string[]
  readonly text: string
}

export type FragmentKind = 'loop' | 'alt' | 'opt' | 'par' | 'critical' | 'break' | 'rect'

export interface FragmentStart {
  readonly type: 'fragment-start'
  readonly kind: FragmentKind
  readonly label: string
}

/** `else`, `and`, `option`: a new section of the open fragment. */
export interface FragmentSection {
  readonly type: 'fragment-section'
  readonly label: string
}

export interface FragmentEnd {
  readonly type: 'fragment-end'
}

export interface Activation {
  readonly type: 'activate' | 'deactivate'
  readonly participantId: string
}

export type SequenceStep = SequenceMessage | SequenceNote | FragmentStart | FragmentSection | FragmentEnd | Activation

export interface SequenceScene {
  readonly participants: readonly SequenceParticipant[]
  readonly boxes: readonly SequenceBox[]
  readonly steps: readonly SequenceStep[]
  readonly autonumber: boolean
}

export function messagesOf(scene: SequenceScene): SequenceMessage[] {
  return scene.steps.filter((step): step is SequenceMessage => step.type === 'message')
}

/** `#3`, the link key of the third message. */
export function messageKey(number: number): string {
  return `#${number}`
}
