import { zoomAt, panBy, type Camera, type Size } from '@/features/diagram/lib/viewport'

export const ZOOM_STEP = 1.25
const PAN_STEP_PX = 80

export type ViewerAction =
  | { readonly kind: 'zoom'; readonly factor: number }
  | { readonly kind: 'pan'; readonly dx: number; readonly dy: number }
  | { readonly kind: 'fit' }
  | { readonly kind: 'close' }

const ACTIONS: Readonly<Record<string, ViewerAction>> = {
  '+': { kind: 'zoom', factor: ZOOM_STEP },
  '=': { kind: 'zoom', factor: ZOOM_STEP },
  '-': { kind: 'zoom', factor: 1 / ZOOM_STEP },
  _: { kind: 'zoom', factor: 1 / ZOOM_STEP },
  '0': { kind: 'fit' },
  f: { kind: 'fit' },
  ArrowLeft: { kind: 'pan', dx: PAN_STEP_PX, dy: 0 },
  ArrowRight: { kind: 'pan', dx: -PAN_STEP_PX, dy: 0 },
  ArrowUp: { kind: 'pan', dx: 0, dy: PAN_STEP_PX },
  ArrowDown: { kind: 'pan', dx: 0, dy: -PAN_STEP_PX },
  Escape: { kind: 'close' }
}

/** What a key does to the enlarged diagram, or null for a key it leaves alone. */
export function actionOfKey(key: string): ViewerAction | null {
  return ACTIONS[key] ?? null
}

/** The camera after a zoom step (around the middle of the viewport) or a pan; any other action leaves it as it is. */
export function cameraAfter(action: ViewerAction, camera: Camera, viewport: Size): Camera {
  if (action.kind === 'zoom') return zoomAt(camera, action.factor, { x: viewport.width / 2, y: viewport.height / 2 })
  if (action.kind === 'pan') return panBy(camera, action.dx, action.dy)
  return camera
}
