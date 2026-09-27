import { inject, provide, ref, type InjectionKey, type Ref } from 'vue'
import { fitInside, interpolate, type Box, type Camera, type Size } from '@/common/diagram/viewport'

/**
 * The one camera a diagram stage shares between its canvas, minimap and toolbar. Moves are
 * eased over a quarter second, or instant under reduced motion.
 */
export interface DiagramCamera {
  readonly camera: Ref<Camera>
  readonly viewport: Ref<Size>
  set: (next: Camera) => void
  animateTo: (next: Camera) => void
  fit: (bounds: Box, animated: boolean) => void
}

const KEY: InjectionKey<DiagramCamera> = Symbol('diagram-camera')
const DURATION_MS = 260
/** What the floating title card, toolbar and corner panels cover, kept clear when fitting. */
const FIT_INSETS = { top: 132, right: 48, bottom: 48, left: 48 }

export function provideDiagramCamera(): DiagramCamera {
  const camera = ref<Camera>({ x: 0, y: 0, k: 1 })
  const viewport = ref<Size>({ width: 0, height: 0 })
  let frame = 0

  function set(next: Camera): void {
    cancelAnimationFrame(frame)
    camera.value = next
  }

  function animateTo(next: Camera): void {
    cancelAnimationFrame(frame)
    const reduced = typeof window !== 'undefined' && window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
    if (reduced) {
      camera.value = next
      return
    }
    const from = camera.value
    const started = performance.now()
    const step = (now: number): void => {
      const t = Math.min(1, (now - started) / DURATION_MS)
      camera.value = interpolate(from, next, t)
      if (t < 1) frame = requestAnimationFrame(step)
    }
    frame = requestAnimationFrame(step)
  }

  function fit(bounds: Box, animated: boolean): void {
    if (viewport.value.width === 0) return
    const target = fitInside(bounds, viewport.value, FIT_INSETS)
    if (animated) animateTo(target)
    else set(target)
  }

  const shared: DiagramCamera = { camera, viewport, set, animateTo, fit }
  provide(KEY, shared)
  return shared
}

export function useDiagramCamera(): DiagramCamera {
  const shared = inject(KEY)
  if (!shared) throw new Error('useDiagramCamera needs a diagram stage above it')
  return shared
}
