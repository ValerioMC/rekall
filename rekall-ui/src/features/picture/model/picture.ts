export type PictureOwnerKind = 'TASK' | 'BLUEPRINT' | 'PAGE' | 'COMPONENT'

/** The record a picture is inserted into: it decides the folder the file is kept in. A blueprint is named by its project's id. */
export interface PictureOwner {
  readonly kind: PictureOwnerKind
  readonly id: string
}

export interface ImportedPicture {
  readonly url: string
}

/** A picture file that arrived by paste or drop. `name` is null for one the clipboard gave no name. */
export interface TransferredPicture {
  readonly file: File
  readonly name: string | null
}
