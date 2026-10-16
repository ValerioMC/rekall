import { onBeforeUnmount, ref, type Ref } from 'vue'
import type { Box, Point } from '@/features/diagram/lib/viewport'
import { FLOW_END, type RouteTarget } from '@/features/flow/lib/route-target'
import type { TaskStepId } from '@/model/branded'

/** Pointer travel under which a press on a step is a click, not a drag. */
const DRAG_THRESHOLD = 4
/** The nearest a step may be dragged to the top left corner of the canvas. */
const EDGE = 8

/** A route being drawn: the step it leaves, where the pointer is, and what it would land on. */
export interface Connecting {
  readonly from: TaskStepId
  readonly at: Point
  readonly over: RouteTarget | null
}

export interface FlowDragOptions {
  /** The element the canvas content is drawn in, which pointer positions are measured against. */
  surface: Ref<HTMLElement | null>
  boxes: () => ReadonlyMap<TaskStepId, Box>
  endBox: () => Box
  /** Saves a dropped step; the live position is kept until it settles. */
  place: (id: TaskStepId, at: Point) => Promise<void>
  connect: (from: TaskStepId, to: RouteTarget) => void
  select: (id: TaskStepId) => void
}

/** The target whose box holds `at`, steps before End, never the step the route leaves. */
export function targetAt(
  at: Point,
  from: TaskStepId,
  boxes: ReadonlyMap<TaskStepId, Box>,
  end: Box
): RouteTarget | null {
  for (const [id, box] of boxes) if (id !== from && holds(box, at)) return id
  return holds(end, at) ? FLOW_END : null
}

function holds(box: Box, at: Point): boolean {
  return at.x >= box.x && at.x <= box.x + box.width && at.y >= box.y && at.y <= box.y + box.height
}

/**
 * Dragging on the flow canvas: a step by its body to move it, from its handle to draw a route to
 * another step or to End. Positions in canvas pixels; the page scrolls, it is not scaled.
 */
export function useFlowDrag(options: FlowDragOptions) {
  const live = ref(new Map<TaskStepId, Point>())
  const connecting = ref<Connecting | null>(null)
  let stop: (() => void) | null = null

  function track(onMove: (event: PointerEvent) => void, onEnd: (event: PointerEvent) => void): void {
    stop?.()
    const finish = (event: PointerEvent): void => {
      stop?.()
      onEnd(event)
    }
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', finish)
    window.addEventListener('pointercancel', finish)
    stop = () => {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', finish)
      window.removeEventListener('pointercancel', finish)
      stop = null
    }
  }

  function startMove(id: TaskStepId, press: PointerEvent): void {
    const origin = options.boxes().get(id)
    if (!origin || press.button !== 0) return
    options.select(id)
    let dragged = false
    track(
      (event) => {
        const dx = event.clientX - press.clientX
        const dy = event.clientY - press.clientY
        if (!dragged && Math.hypot(dx, dy) < DRAG_THRESHOLD) return
        dragged = true
        live.value = new Map(live.value).set(id, {
          x: Math.max(EDGE, Math.round(origin.x + dx)),
          y: Math.max(EDGE, Math.round(origin.y + dy))
        })
      },
      () => {
        const at = live.value.get(id)
        if (!dragged || !at) return
        void options.place(id, at).finally(() => {
          const next = new Map(live.value)
          next.delete(id)
          live.value = next
        })
      }
    )
  }

  function startConnect(from: TaskStepId, press: PointerEvent): void {
    if (press.button !== 0) return
    options.select(from)
    const pointAt = (event: PointerEvent): Point => {
      const rect = options.surface.value?.getBoundingClientRect()
      return { x: event.clientX - (rect?.left ?? 0), y: event.clientY - (rect?.top ?? 0) }
    }
    const aim = (event: PointerEvent): Connecting => {
      const at = pointAt(event)
      return { from, at, over: targetAt(at, from, options.boxes(), options.endBox()) }
    }
    connecting.value = aim(press)
    track(
      (event) => (connecting.value = aim(event)),
      (event) => {
        const over = aim(event).over
        connecting.value = null
        if (over) options.connect(from, over)
      }
    )
  }

  /** Moves a step by a keyboard nudge, from where it stands now. */
  function nudge(id: TaskStepId, dx: number, dy: number): Promise<void> {
    const box = options.boxes().get(id)
    if (!box) return Promise.resolve()
    return options.place(id, { x: Math.max(EDGE, box.x + dx), y: Math.max(EDGE, box.y + dy) })
  }

  onBeforeUnmount(() => stop?.())

  return { live, connecting, startMove, startConnect, nudge }
}
