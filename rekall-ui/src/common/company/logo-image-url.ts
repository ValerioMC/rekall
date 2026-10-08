import { env } from '@/common/config/env'
import { densitySrcset } from '@/common/project/icon-image-url'

/** A company logo rendered by the server at exactly `pixels` per side; `version` makes the URL
 *  name one image for good, so the browser keeps it. */
export function companyLogoImageUrl(companyId: string, version: string, pixels: number): string {
  return `${env.VITE_API_BASE_URL}/api/companies/${companyId}/logo-image?size=${pixels}&v=${encodeURIComponent(version)}`
}

export function companyLogoImageSrcset(companyId: string, version: string, cssSize: number): string {
  return densitySrcset((pixels) => companyLogoImageUrl(companyId, version, pixels), cssSize)
}

/** The letter a company without a logo wears: its name's first character, whole even outside ASCII. */
export function companyInitial(name: string): string {
  return (Array.from(name.trim())[0] ?? '?').toLocaleUpperCase()
}
