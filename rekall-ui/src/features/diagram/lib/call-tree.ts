import { messagesOf, type SequenceMessage, type SequenceScene } from './scene/sequence-scene'

/**
 * Who calls whom inside a sequence, as a tree. A solid message is a call and nests under the
 * call its sender is answering; a dashed one is the reply that closes the call it answers, and
 * every call opened inside it. A self-call, and a reply answering nothing open, are leaves.
 */
export interface CallNode {
  readonly call: SequenceMessage
  readonly reply: SequenceMessage | null
  readonly children: readonly CallNode[]
}

interface OpenCall {
  readonly call: SequenceMessage
  reply: SequenceMessage | null
  readonly children: OpenCall[]
}

interface Frame {
  /** Who is answering this call: the receiver of `node.call`. */
  readonly participantId: string
  readonly node: OpenCall
}

export function callTreeOf(scene: SequenceScene): CallNode[] {
  const roots: OpenCall[] = []
  const stack: Frame[] = []
  for (const message of messagesOf(scene)) {
    if (message.line === 'dashed' && closeAnsweredCall(stack, message)) continue
    const node: OpenCall = { call: message, reply: null, children: [] }
    const caller = innermostFrameOf(stack, message.from)
    stack.length = caller + 1
    if (caller < 0) roots.push(node)
    else stack[caller]!.node.children.push(node)
    if (message.line === 'solid' && message.from !== message.to) stack.push({ participantId: message.to, node })
  }
  return roots
}

/** The deepest open call `participantId` is answering, or -1 when it is answering none. */
function innermostFrameOf(stack: readonly Frame[], participantId: string): number {
  for (let index = stack.length - 1; index >= 0; index -= 1) if (stack[index]!.participantId === participantId) return index
  return -1
}

/** A reply from the callee back to its caller closes that call and everything above it. */
function closeAnsweredCall(stack: Frame[], reply: SequenceMessage): boolean {
  for (let index = stack.length - 1; index >= 0; index -= 1) {
    const frame = stack[index]!
    if (frame.participantId === reply.from && frame.node.call.from === reply.to) {
      frame.node.reply = reply
      stack.length = index
      return true
    }
  }
  return false
}

/** The outermost calls a participant receives: where its part in the sequence starts. */
export function callsInto(roots: readonly CallNode[], participantId: string): CallNode[] {
  return roots.flatMap((node) => (node.call.to === participantId && node.call.from !== participantId ? [node] : callsInto(node.children, participantId)))
}

/** The call a message opens or answers. */
export function callOf(roots: readonly CallNode[], number: number): CallNode | null {
  for (const node of roots) {
    if (node.call.number === number || node.reply?.number === number) return node
    const inside = callOf(node.children, number)
    if (inside) return inside
  }
  return null
}

/** Every message number under `node`, its own call and reply included. */
export function messageNumbersIn(node: CallNode): number[] {
  const own = node.reply ? [node.call.number, node.reply.number] : [node.call.number]
  return [...own, ...node.children.flatMap(messageNumbersIn)]
}
