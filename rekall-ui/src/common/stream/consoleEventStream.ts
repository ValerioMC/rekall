import { env } from '@/common/config/env'

export type FrameListener = (event: Event) => void

const listenersByFrame = new Map<string, Set<FrameListener>>()
const READY_STATE_OPEN = 1
let source: EventSource | null = null

/**
 * The one `EventSource` on `/api/steps/stream` that every feed of the console shares. A webview
 * keeps at most six HTTP/1.1 connections per host, so a stream per feed starves the ordinary
 * requests queued behind them until they time out. It opens with the first subscriber and
 * closes with the last.
 */
export function subscribeToFrame(frameName: string, listener: FrameListener): () => void {
  if (typeof EventSource === 'undefined') return () => {}
  const stream = source ?? openStream()
  const listeners = listenersByFrame.get(frameName) ?? listenToFrame(stream, frameName)
  listeners.add(listener)
  return () => unsubscribe(frameName, listener)
}

export function isConsoleStreamOpen(): boolean {
  return source !== null && source.readyState === READY_STATE_OPEN
}

function openStream(): EventSource {
  source = new EventSource(`${env.VITE_API_BASE_URL}/api/steps/stream`)
  return source
}

function listenToFrame(stream: EventSource, frameName: string): Set<FrameListener> {
  const listeners = new Set<FrameListener>()
  listenersByFrame.set(frameName, listeners)
  stream.addEventListener(frameName, (event) => {
    for (const listener of [...listeners]) listener(event)
  })
  return listeners
}

function unsubscribe(frameName: string, listener: FrameListener): void {
  listenersByFrame.get(frameName)?.delete(listener)
  if (hasSubscribers()) return
  source?.close()
  source = null
  listenersByFrame.clear()
}

function hasSubscribers(): boolean {
  return [...listenersByFrame.values()].some((listeners) => listeners.size > 0)
}
