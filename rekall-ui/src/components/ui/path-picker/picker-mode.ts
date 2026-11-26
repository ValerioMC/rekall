import { isPictureFileName } from '@/features/picture/lib/picture-file'
import type { DirectoryEntry } from '@/model/filesystem'

/** What the picker is choosing: a path to write, a folder, or a picture file to copy. */
export type PickerMode = 'path' | 'folder' | 'image'

/** What the footer says ↵ and the preview stand for. */
export interface PickerWords {
  readonly preview: string
  readonly enter: string
}

export interface PickerModeRules {
  readonly title: string
  readonly placeholder: string
  readonly listLabel: string
  readonly lists: (entry: DirectoryEntry) => boolean
  /** False when a folder is only the way to a file: ↵ opens it. */
  readonly choosesFolders: boolean
  readonly words: (current: DirectoryEntry | null) => PickerWords
}

const ANY_PATH = 'Filter, or type a path: src/  ~/  /'
const EVERYTHING_LABEL = 'Folders and files'

export const PICKER_MODES: Readonly<Record<PickerMode, PickerModeRules>> = {
  path: {
    title: 'Insert a path',
    placeholder: ANY_PATH,
    listLabel: EVERYTHING_LABEL,
    lists: () => true,
    choosesFolders: true,
    words: () => ({ preview: 'Inserts', enter: 'insert' })
  },
  folder: {
    title: 'Choose a folder',
    placeholder: ANY_PATH,
    listLabel: EVERYTHING_LABEL,
    lists: (entry) => entry.directory,
    choosesFolders: true,
    words: () => ({ preview: 'Chooses', enter: 'choose' })
  },
  image: {
    title: 'Choose an image',
    placeholder: 'Filter, or type a path: ~/Desktop/  /',
    listLabel: 'Folders and pictures',
    lists: (entry) => entry.directory || isPictureFileName(entry.name),
    choosesFolders: false,
    words: (current) => {
      if (current?.directory) return { preview: 'Opens', enter: 'open' }
      return { preview: current ? 'Copies' : 'Copies nothing yet', enter: 'choose' }
    }
  }
}
