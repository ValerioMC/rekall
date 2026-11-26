/** A picture no text shows. `url` is the address markdown would show it with, `folder` its place under the images folder. */
export interface UnusedPicture {
  readonly url: string
  readonly folder: string
  readonly sizeBytes: number
}

/** Where the one cleanup of unused pictures stands, with what each state has to show. */
export type PictureCleanupStatus =
  | { readonly state: 'IDLE' }
  | { readonly state: 'SEARCHING'; readonly percentage: number }
  | { readonly state: 'READY'; readonly pictures: readonly UnusedPicture[]; readonly totalBytes: number }
  | { readonly state: 'DELETING'; readonly percentage: number }
  | {
      readonly state: 'FINISHED'
      readonly removed: number
      readonly freedBytes: number
      readonly kept: number
      readonly notDeleted: number
    }
  | { readonly state: 'FAILED'; readonly message: string }

export type PictureCleanupState = PictureCleanupStatus['state']

/** One frame of a run. `count` is the unused pictures of a ready list and the removed ones of a finished run. */
export interface PictureCleanupStreamEvent {
  readonly state: PictureCleanupState
  readonly percentage: number
  readonly count: number
}
