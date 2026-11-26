const PICTURE_EXTENSION = /\.(png|jpe?g|gif|webp|svg)$/i

/** What a picture the clipboard gave no name is kept and described as. */
export const PASTED_PICTURE_NAME = 'pasted-image'

/** Shown when a copy ends after the text it was meant for has gone. */
export const PICTURE_NOT_INSERTED = 'The text changed while the picture was being copied, so it was not inserted.'

export function isPictureFileName(name: string): boolean {
  return PICTURE_EXTENSION.test(name)
}

function nameWithoutExtension(path: string): string {
  const fileName = path.slice(Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\')) + 1)
  return fileName.replace(PICTURE_EXTENSION, '')
}

/** The markdown of a picture, described by the name of the file it was copied from. */
export function pictureMarkdown(sourcePath: string, url: string): string {
  const description = nameWithoutExtension(sourcePath).replace(/[\\[\]]/g, '\\$&')
  return `![${description}](${url})`
}
