import type { PictureAlignment } from '@/features/picture/model/picture'

export const PICTURE_ALIGNMENTS: readonly PictureAlignment[] = ['left', 'center', 'right']

export const PICTURE_ALIGNMENT_LABEL: Readonly<Record<PictureAlignment, string>> = {
  left: 'Left',
  center: 'Centre',
  right: 'Right'
}

/** The end of an address, before the `>` of one written `<like this>`. */
const ALIGNMENT_ENDING = /(?:#(left|center|right))?(>?)$/

/** How the picture at `address` is drawn: its ending says so, and none means left. */
export function alignmentOf(address: string): PictureAlignment {
  return (ALIGNMENT_ENDING.exec(address)?.[1] as PictureAlignment | undefined) ?? 'left'
}

export function alignedAddress(address: string, alignment: PictureAlignment): string {
  const ending = alignment === 'left' ? '' : `#${alignment}`
  return address.replace(ALIGNMENT_ENDING, (_all, _previous, bracket: string) => `${ending}${bracket}`)
}
