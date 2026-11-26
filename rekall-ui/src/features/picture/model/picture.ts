export type PictureOwnerKind = 'TASK' | 'BLUEPRINT' | 'PAGE' | 'COMPONENT'

/** The record a picture is inserted into: it decides the folder the file is kept in. A blueprint is named by its project's id. */
export interface PictureOwner {
  readonly kind: PictureOwnerKind
  readonly id: string
}

export interface ImportedPicture {
  readonly url: string
}
