import { SECTION_BODY_MAX_CHARACTERS } from '@/features/project/lib/section-size'

const TITLE_MAX_CHARACTERS = 200
const TOP_HEADING = /^#\s+(.+?)\s*#*\s*$/
const FENCE = /^\s*(```|~~~)/
const EXTENSION = /\.(md|markdown|txt)$/i

export interface SectionFile {
  readonly title: string
  readonly body: string
}

/** A section made of a markdown file: titled by its first `#` heading, else by its file name. Refuses an empty file or one over the size a section holds. */
export function sectionOfFile(fileName: string, text: string): SectionFile {
  if (text.trim() === '') throw new Error(`${fileName} is empty.`)
  if (text.length > SECTION_BODY_MAX_CHARACTERS) {
    throw new Error(
      `${fileName} has ${text.length.toLocaleString('en-US')} characters; a section holds ${SECTION_BODY_MAX_CHARACTERS.toLocaleString('en-US')}. Nothing was imported.`
    )
  }
  const title = (topHeadingOf(text) ?? fileName.replace(EXTENSION, '')).slice(0, TITLE_MAX_CHARACTERS)
  return { title, body: text }
}

function topHeadingOf(text: string): string | null {
  let inFence = false
  for (const line of text.split('\n')) {
    if (FENCE.test(line)) inFence = !inFence
    const found = inFence ? null : TOP_HEADING.exec(line)
    if (found) return found[1]!
  }
  return null
}
