import { onScopeDispose, ref } from 'vue'
import { env } from '@/common/config/env'
import type { TerminalEndedFrame } from '@/model/terminal'
import type { TerminalId } from '@/model/branded'

export interface TerminalSocketHandlers {
  onOutput: (bytes: Uint8Array) => void
  onReady: (info: { anchors: string; workingDir: string }) => void
  onEnded: (frame: TerminalEndedFrame) => void
  onClose: () => void
}

export interface TerminalSocket {
  connected: ReturnType<typeof ref<boolean>>
  connect: (id: TerminalId) => void
  disconnect: () => void
  sendInput: (data: string | Uint8Array) => void
  sendResize: (cols: number, rows: number) => void
}

/**
 * The one WebSocket a terminal pane holds: binary frames both ways for PTY bytes, a
 * `{"resize":[cols,rows]}` text frame for size, a `{"type":"ended"}` frame on exit. One connection
 * at a time; `connect` closes the previous, the calling scope closes the last.
 */
export function useTerminalSocket(handlers: TerminalSocketHandlers): TerminalSocket {
  const connected = ref(false)
  const encoder = new TextEncoder()
  let socket: WebSocket | null = null

  function socketUrl(id: TerminalId): string {
    const base = env.VITE_API_BASE_URL
    if (base) return `${base.replace(/^http/, 'ws')}/api/terminal/${id}/io`
    const scheme = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    return `${scheme}//${window.location.host}/api/terminal/${id}/io`
  }

  function disconnect(): void {
    if (socket) {
      socket.onclose = null
      socket.onerror = null
      socket.onmessage = null
      socket.onopen = null
      if (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING) {
        socket.close()
      }
      socket = null
    }
    connected.value = false
  }

  function connect(id: TerminalId): void {
    disconnect()
    if (typeof WebSocket === 'undefined') return
    const next = new WebSocket(socketUrl(id))
    next.binaryType = 'arraybuffer'
    socket = next

    next.onopen = () => {
      connected.value = true
    }
    next.onerror = () => {
      connected.value = next.readyState === WebSocket.OPEN
    }
    next.onclose = () => {
      if (socket === next) {
        connected.value = false
        socket = null
        handlers.onClose()
      }
    }
    next.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        handlers.onOutput(new Uint8Array(event.data))
        return
      }
      let frame: unknown
      try {
        frame = JSON.parse(String(event.data))
      } catch {
        return
      }
      if (!frame || typeof frame !== 'object') return
      const record = frame as Record<string, unknown>
      if (record.type === 'ready') {
        handlers.onReady({
          anchors: String(record.anchors ?? ''),
          workingDir: String(record.workingDir ?? '')
        })
      } else if (record.type === 'ended') {
        handlers.onEnded({
          type: 'ended',
          exitCode: Number(record.exitCode ?? 0),
          detail: String(record.detail ?? '')
        })
      }
    }
  }

  function sendInput(data: string | Uint8Array): void {
    if (!socket || socket.readyState !== WebSocket.OPEN) return
    socket.send(typeof data === 'string' ? encoder.encode(data) : data)
  }

  function sendResize(cols: number, rows: number): void {
    if (!socket || socket.readyState !== WebSocket.OPEN) return
    socket.send(JSON.stringify({ resize: [Math.round(cols), Math.round(rows)] }))
  }

  onScopeDispose(disconnect)

  return { connected, connect, disconnect, sendInput, sendResize }
}
