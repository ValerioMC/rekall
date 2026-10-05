import { env } from '@/common/config/env'

/** The screen densities a mark is prepared for; the browser picks the one it has. */
const DENSITIES = [1, 2, 3] as const

/** An icon picture rendered by the server at exactly `pixels` per side; `version` makes the URL
 *  name one image for good, so the browser keeps it. */
export function projectIconImageUrl(projectId: string, version: string, pixels: number): string {
  return `${env.VITE_API_BASE_URL}/api/projects/${projectId}/icon-image?size=${pixels}&v=${encodeURIComponent(version)}`
}

/** One candidate per density, each sized in device pixels so no spot ever scales the image. */
export function projectIconImageSrcset(projectId: string, version: string, cssSize: number): string {
  return DENSITIES.map(
    (density) => `${projectIconImageUrl(projectId, version, Math.round(cssSize * density))} ${density}x`
  ).join(', ')
}
