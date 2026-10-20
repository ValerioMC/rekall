/**
 * Where a reader is in a walkthrough: a stack of frames, the sheet that started it at the
 * bottom and every call stepped into above it. Pure: each move returns the next stack, and a
 * frame's position is clamped only once its sheet is drawn and its beats counted.
 */

export interface WalkFrame {
  readonly sheetIndex: number
  readonly position: number
  /** The text of the call that opened this frame; none at the bottom. */
  readonly via: string | null
}

/** Never empty: every move keeps the frame the walkthrough started on. */
export type WalkStack = readonly WalkFrame[]

export function startAt(sheetIndex: number): WalkStack {
  return [{ sheetIndex, position: 0, via: null }]
}

export function topOf(stack: WalkStack): WalkFrame {
  return stack[stack.length - 1]!
}

export function moveTo(stack: WalkStack, position: number, beatCount: number): WalkStack {
  const clamped = Math.max(0, Math.min(position, beatCount - 1))
  return replaceTop(stack, { ...topOf(stack), position: clamped })
}

/** One beat on; past the end of a stepped-into call, back to the caller's next beat. Null at the very end. */
export function advance(stack: WalkStack, beatCount: number): WalkStack | null {
  const top = topOf(stack)
  if (top.position < beatCount - 1) return replaceTop(stack, { ...top, position: top.position + 1 })
  return stack.length > 1 ? leave(stack, true) : null
}

/** One beat back; before the start of a stepped-into call, back to the call itself. Null at the very start. */
export function retreat(stack: WalkStack): WalkStack | null {
  const top = topOf(stack)
  if (top.position > 0) return replaceTop(stack, { ...top, position: top.position - 1 })
  return stack.length > 1 ? leave(stack, false) : null
}

export function enter(stack: WalkStack, sheetIndex: number, via: string): WalkStack {
  return [...stack, { sheetIndex, position: 0, via }]
}

/** Back to the caller, on the call itself or on the beat after it. */
export function leave(stack: WalkStack, onward: boolean): WalkStack {
  if (stack.length === 1) return stack
  const rest = stack.slice(0, -1)
  const caller = topOf(rest)
  return onward ? replaceTop(rest, { ...caller, position: caller.position + 1 }) : rest
}

/** Back out to the frame at `depth`, where the reader left it. */
export function leaveTo(stack: WalkStack, depth: number): WalkStack {
  return stack.slice(0, Math.max(1, depth + 1))
}

function replaceTop(stack: WalkStack, frame: WalkFrame): WalkStack {
  return [...stack.slice(0, -1), frame]
}
