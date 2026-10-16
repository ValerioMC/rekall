/**
 * The camera over the canvas: a translation and a scale, screen = world × k + (x, y). Pure, so
 * zoom-around-the-pointer and fit-to-content are tested without a DOM.
 */

export interface Camera {
  readonly x: number
  readonly y: number
  readonly k: number
}

export interface Box {
  readonly x: number
  readonly y: number
  readonly width: number
  readonly height: number
}

export interface Size {
  readonly width: number
  readonly height: number
}

export interface Point {
  readonly x: number
  readonly y: number
}

export const MIN_SCALE = 0.15
export const MAX_SCALE = 2.5
/** Fitting never magnifies past this: a three-node diagram should not fill a 30" screen. */
const FIT_MAX_SCALE = 1.1

export function clampScale(k: number): number {
  return Math.min(MAX_SCALE, Math.max(MIN_SCALE, k))
}

/** Scales by `factor` keeping the world point under `anchor` (screen coordinates) still. */
export function zoomAt(camera: Camera, factor: number, anchor: Point): Camera {
  const k = clampScale(camera.k * factor)
  const ratio = k / camera.k
  return { k, x: anchor.x - (anchor.x - camera.x) * ratio, y: anchor.y - (anchor.y - camera.y) * ratio }
}

export function panBy(camera: Camera, dx: number, dy: number): Camera {
  return { ...camera, x: camera.x + dx, y: camera.y + dy }
}

/** The camera that shows all of `bounds` inside `viewport`, `padding` pixels clear of its edges. */
export function fitCamera(bounds: Box, viewport: Size, padding: number): Camera {
  const availableWidth = Math.max(1, viewport.width - padding * 2)
  const availableHeight = Math.max(1, viewport.height - padding * 2)
  const k = clampScale(Math.min(FIT_MAX_SCALE, availableWidth / Math.max(1, bounds.width), availableHeight / Math.max(1, bounds.height)))
  return {
    k,
    x: (viewport.width - bounds.width * k) / 2 - bounds.x * k,
    y: (viewport.height - bounds.height * k) / 2 - bounds.y * k
  }
}

export interface Insets {
  readonly top: number
  readonly right: number
  readonly bottom: number
  readonly left: number
}

/** {@link fitCamera} into the part of the viewport the overlays leave clear. */
export function fitInside(bounds: Box, viewport: Size, insets: Insets): Camera {
  const inner = { width: viewport.width - insets.left - insets.right, height: viewport.height - insets.top - insets.bottom }
  const camera = fitCamera(bounds, inner, 0)
  return { k: camera.k, x: camera.x + insets.left, y: camera.y + insets.top }
}

/** Keeps the scale and puts `point` (world) at the centre of the viewport. */
export function centreOn(camera: Camera, point: Point, viewport: Size): Camera {
  return { k: camera.k, x: viewport.width / 2 - point.x * camera.k, y: viewport.height / 2 - point.y * camera.k }
}

/** Whether `box` (world) lies wholly inside the viewport, `margin` pixels in. */
export function isInView(camera: Camera, box: Box, viewport: Size, margin: number): boolean {
  const left = box.x * camera.k + camera.x
  const top = box.y * camera.k + camera.y
  return (
    left >= margin &&
    top >= margin &&
    left + box.width * camera.k <= viewport.width - margin &&
    top + box.height * camera.k <= viewport.height - margin
  )
}

/** Eases from one camera to another; `t` in 0..1. */
export function interpolate(from: Camera, to: Camera, t: number): Camera {
  const eased = 1 - Math.pow(1 - t, 3)
  return {
    x: from.x + (to.x - from.x) * eased,
    y: from.y + (to.y - from.y) * eased,
    k: from.k + (to.k - from.k) * eased
  }
}
