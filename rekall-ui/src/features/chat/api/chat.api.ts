import { apiClient, request } from '../../../api/client'
import { ChatEventSchema } from './schemas/chat-event.schema'
import { env } from '@/common/config/env'
import type { ChatEvent } from '../model/chat-event'
import type { TerminalId } from '@/model/branded'

export async function sendMessage(id: TerminalId, text: string): Promise<void> {
  await request(() => apiClient(`/api/chat/${id}/messages`, { method: 'POST', body: { text } }))
}

export async function interrupt(id: TerminalId): Promise<void> {
  await request(() => apiClient(`/api/chat/${id}/interrupt`, { method: 'POST' }))
}

export async function answerPermission(
  id: TerminalId,
  requestId: string,
  allow: boolean
): Promise<void> {
  await request(() =>
    apiClient(`/api/chat/${id}/permissions/${encodeURIComponent(requestId)}`, {
      method: 'POST',
      body: { allow }
    })
  )
}

/** Where a session's events stream from; a reconnecting `EventSource` resumes with `Last-Event-ID` instead of `after`. */
export function chatEventsUrl(id: TerminalId, after: number): string {
  return `${env.VITE_API_BASE_URL}/api/chat/${id}/events?after=${after}`
}

/** The event a stream message carries, or none for one this console does not know or cannot read. */
export function parseChatEvent(data: string): ChatEvent | null {
  try {
    const parsed = ChatEventSchema.safeParse(JSON.parse(data))
    return parsed.success ? parsed.data : null
  } catch {
    return null
  }
}
