import { describe, expect, it } from 'vitest'
import {
  filterEntries,
  formatPathForMarkdown,
  isInsideCode,
  matchName,
  splitByRanges,
  splitPathQuery,
  withFolderSlash
} from '@/common/path/path-query'
import type { DirectoryEntry } from '@/model/filesystem'

function entry(name: string, directory = false, hidden = name.startsWith('.')): DirectoryEntry {
  return { name, path: `/p/${name}`, directory, hidden }
}

describe('splitPathQuery', () => {
  it('filters the current folder when nothing names another one', () => {
    expect(splitPathQuery('comp')).toEqual({ folder: '', needle: 'comp' })
    expect(splitPathQuery('')).toEqual({ folder: '', needle: '' })
  })

  it('splits at the last slash into a folder to list and a name to filter by', () => {
    expect(splitPathQuery('src/comp')).toEqual({ folder: 'src/', needle: 'comp' })
    expect(splitPathQuery('~/Downl')).toEqual({ folder: '~/', needle: 'Downl' })
    expect(splitPathQuery('/')).toEqual({ folder: '/', needle: '' })
  })

  it('reads ~ and .. alone as folders, since nothing is named either', () => {
    expect(splitPathQuery('~')).toEqual({ folder: '~', needle: '' })
    expect(splitPathQuery('..')).toEqual({ folder: '..', needle: '' })
  })
})

describe('matchName', () => {
  it('ranks a prefix over a substring over scattered letters', () => {
    const prefix = matchName('components', 'comp')!
    const substring = matchName('ui-components', 'comp')!
    const scattered = matchName('markdown-editor.ts', 'mdedt')!

    expect(prefix.score).toBeLessThan(substring.score)
    expect(substring.score).toBeLessThan(scattered.score)
    expect(prefix.ranges).toEqual([[0, 4]])
    expect(substring.ranges).toEqual([[3, 7]])
  })

  it('merges adjacent scattered letters into one range and ignores case', () => {
    expect(matchName('ReadMe.md', 'RME')!.ranges).toEqual([
      [0, 1],
      [4, 6]
    ])
  })

  it('refuses letters that are not all there, in order', () => {
    expect(matchName('app.ts', 'tsa')).toBeNull()
  })
})

describe('filterEntries', () => {
  const entries = [entry('src', true), entry('.git', true), entry('scripts', true), entry('.env'), entry('setup.ts')]

  it('keeps the server order and hides dotfiles with no needle', () => {
    expect(filterEntries(entries, '', false).map((row) => row.entry.name)).toEqual([
      'src',
      'scripts',
      'setup.ts'
    ])
    expect(filterEntries(entries, '', true)).toHaveLength(5)
  })

  it('shows hidden entries when the needle starts with a dot', () => {
    expect(filterEntries(entries, '.e', false).map((row) => row.entry.name)).toEqual(['.env'])
  })

  it('puts the best match first and a folder ahead of a file on a tie', () => {
    const tied = [entry('setup.ts'), entry('setup', true)]
    expect(filterEntries(tied, 'set', false).map((row) => row.entry.name)).toEqual(['setup', 'setup.ts'])
    expect(filterEntries(entries, 'sc', false).map((row) => row.entry.name)).toEqual(['scripts', 'src'])
  })
})

describe('splitByRanges', () => {
  it('cuts a name into matched and unmatched runs', () => {
    expect(splitByRanges('components', [[0, 4]])).toEqual([
      { text: 'comp', matched: true },
      { text: 'onents', matched: false }
    ])
    expect(splitByRanges('abc', [])).toEqual([{ text: 'abc', matched: false }])
  })
})

describe('withFolderSlash', () => {
  it('ends a folder in its separator once, and leaves a file alone', () => {
    expect(withFolderSlash('/a/src', true)).toBe('/a/src/')
    expect(withFolderSlash('/', true)).toBe('/')
    expect(withFolderSlash('C:\\work\\src', true)).toBe('C:\\work\\src\\')
    expect(withFolderSlash('/a/app.ts', false)).toBe('/a/app.ts')
  })
})

describe('isInsideCode', () => {
  it('sees an open fence and an open inline span', () => {
    expect(isInsideCode('text\n```bash\ncd ')).toBe(true)
    expect(isInsideCode('text\n```\ncode\n```\nafter ')).toBe(false)
    expect(isInsideCode('run `cat ')).toBe(true)
    expect(isInsideCode('run `cat` on ')).toBe(false)
    expect(isInsideCode('an escaped \\` tick ')).toBe(false)
  })
})

describe('formatPathForMarkdown', () => {
  it('wraps a path as inline code so underscores survive the preview', () => {
    expect(formatPathForMarkdown('/a/my_file_name.md', false, 'See ')).toBe('`/a/my_file_name.md`')
    expect(formatPathForMarkdown('/a/src', true, 'See ')).toBe('`/a/src/`')
  })

  it('goes in bare where the caret is already inside code', () => {
    expect(formatPathForMarkdown('/a/src', true, 'Run `ls ')).toBe('/a/src/')
    expect(formatPathForMarkdown('/a/x.sh', false, '```\n')).toBe('/a/x.sh')
  })

  it('uses double backticks for a path holding one', () => {
    expect(formatPathForMarkdown('/a/we`ird', false, '')).toBe('`` /a/we`ird ``')
  })
})
