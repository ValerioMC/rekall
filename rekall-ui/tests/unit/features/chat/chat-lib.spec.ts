import { describe, expect, it } from 'vitest'
import { describeTool } from '@/features/chat/lib/describe-tool'
import { splitLines } from '@/features/chat/lib/lines'
import { diffLines, writeLines } from '@/features/chat/lib/tool-diff'

describe('splitLines', () => {
  it('has no lines for no text, and does not count the newline that ends the text', () => {
    expect(splitLines('')).toEqual([])
    expect(splitLines('a\nb\n')).toEqual(['a', 'b'])
    expect(splitLines('a\n\nb')).toEqual(['a', '', 'b'])
  })
})

describe('diffLines', () => {
  it('shows what an edit takes out, then what it puts in', () => {
    expect(diffLines('let a = 1\nlet b = 2', 'let a = 3')).toEqual([
      { sign: '-', text: 'let a = 1' },
      { sign: '-', text: 'let b = 2' },
      { sign: '+', text: 'let a = 3' }
    ])
  })

  it('shows a file that is written as all added', () => {
    expect(writeLines('one\ntwo\n')).toEqual([
      { sign: '+', text: 'one' },
      { sign: '+', text: 'two' }
    ])
  })
})

describe('describeTool', () => {
  it('names a command by its first line', () => {
    expect(describeTool({ tool: 'bash', command: 'cargo test\n--workspace', description: null })).toEqual({
      name: 'Bash',
      target: 'cargo test',
      icon: 'terminal'
    })
  })

  it('keeps the end of a long path, where the file is', () => {
    const path = `/Users/someone/Projects/${'deep/'.repeat(12)}file.rs`

    const { target } = describeTool({ tool: 'read', filePath: path })

    expect(target.length).toBe(48)
    expect(target.startsWith('…')).toBe(true)
    expect(target.endsWith('file.rs')).toBe(true)
  })

  it('leaves a short path whole', () => {
    expect(describeTool({ tool: 'edit', filePath: 'src/main.rs', oldString: 'a', newString: 'b', replaceAll: false }).target).toBe(
      'src/main.rs'
    )
  })

  it('names a search by its pattern and where it looks', () => {
    expect(describeTool({ tool: 'search', name: 'Grep', pattern: 'fn main', path: 'src' })).toEqual({
      name: 'Grep',
      target: 'fn main in src',
      icon: 'search'
    })
    expect(describeTool({ tool: 'search', name: 'Glob', pattern: '**/*.rs', path: null }).target).toBe('**/*.rs')
  })

  it('names any other tool by itself', () => {
    expect(describeTool({ tool: 'other', name: 'WebFetch', text: 'https://example.com\nmore' })).toEqual({
      name: 'WebFetch',
      target: 'https://example.com',
      icon: 'tool'
    })
  })
})
