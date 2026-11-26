import { onScopeDispose } from 'vue'
import { subscribeToFrame } from '@/common/stream/consoleEventStream'
import { PictureCleanupStreamEventSchema } from '@/features/picture/api/schemas/picture-cleanup.schema'
import type { PictureCleanupStreamEvent } from '@/features/picture/model/picture-cleanup'

function parsed(event: Event): unknown {
  try {
    return JSON.parse((event as MessageEvent<string>).data)
  } catch {
    return null
  }
}

/** The `picture-cleanup` frames of the console's event stream: a run's state and percentage as they change. */
export function usePictureCleanupStream(onFrame: (event: PictureCleanupStreamEvent) => void): void {
  const unsubscribe = subscribeToFrame('picture-cleanup', (event) => {
    const frame = PictureCleanupStreamEventSchema.safeParse(parsed(event))
    if (frame.success) onFrame(frame.data)
  })
  onScopeDispose(unsubscribe)
}
