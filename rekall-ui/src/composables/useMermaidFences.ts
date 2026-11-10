import { nextTick, onBeforeUnmount, onMounted, type Ref } from 'vue'
import { describeMermaidFailure, MERMAID_FENCE_CLASS, MERMAID_FENCE_FAILED_CLASS, sourceOfMermaidFrame } from '@/common/markdown/mermaid-fence'

/**
 * Draws every mermaid frame of a read view as the Diagram section draws a sheet, and takes each
 * drawing down when its frame leaves the page or the view goes. A frame already drawn is left alone.
 */
export function useMermaidFences(root: Ref<HTMLElement | null>): { paint: () => void } {
  const lifetimes = new Map<HTMLElement, AbortController>()

  function takeDownDetached(): void {
    for (const [frame, lifetime] of lifetimes) {
      if (frame.isConnected) continue
      lifetime.abort()
      lifetimes.delete(frame)
    }
  }

  async function draw(frame: HTMLElement, lifetime: AbortController): Promise<void> {
    try {
      const { showStaticDiagram } = await import('@/features/diagram/lib/static-diagram')
      await showStaticDiagram(frame, sourceOfMermaidFrame(frame), lifetime.signal)
    } catch (failure: unknown) {
      frame.classList.add(MERMAID_FENCE_FAILED_CLASS)
      frame.textContent = describeMermaidFailure(failure)
    }
  }

  function paint(): void {
    void nextTick(() => {
      takeDownDetached()
      for (const frame of root.value?.querySelectorAll<HTMLElement>(`.${MERMAID_FENCE_CLASS}`) ?? []) {
        if (lifetimes.has(frame)) continue
        const lifetime = new AbortController()
        lifetimes.set(frame, lifetime)
        void draw(frame, lifetime)
      }
    })
  }

  onMounted(paint)
  onBeforeUnmount(() => {
    lifetimes.forEach((lifetime) => lifetime.abort())
    lifetimes.clear()
  })

  return { paint }
}
