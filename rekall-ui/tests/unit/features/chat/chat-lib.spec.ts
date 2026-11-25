import { describe, expect, it } from 'vitest'
import { mcpArguments } from '@/features/chat/lib/mcp-arguments'
import { describePermission } from '@/features/chat/lib/describe-permission'
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

describe('describeTool for the tools the terminal draws in their own way', () => {
  it('counts the todos that are done', () => {
    const items = [
      { content: 'a', status: 'completed' as const },
      { content: 'b', status: 'inProgress' as const },
      { content: 'c', status: 'pending' as const }
    ]

    expect(describeTool({ tool: 'todos', items })).toEqual({ name: 'Todos', target: '1 of 3 done', icon: 'checklist' })
  })

  it('names an agent by what it was asked to do', () => {
    expect(describeTool({ tool: 'agent', description: 'Find the parser', subagentType: 'Explore', prompt: 'p' })).toEqual({
      name: 'Agent',
      target: 'Find the parser',
      icon: 'agent'
    })
  })

  it('names a web call by the host it fetches or the query it searches', () => {
    expect(describeTool({ tool: 'web', name: 'WebFetch', target: 'https://example.com/docs/page' }).target).toBe('example.com')
    expect(describeTool({ tool: 'web', name: 'WebSearch', target: 'rust serde tag' })).toEqual({
      name: 'WebSearch',
      target: 'rust serde tag',
      icon: 'web'
    })
  })

  it('names an mcp call by its server and tool', () => {
    expect(describeTool({ tool: 'mcp', server: 'rekall', toolName: 'rekall_context', text: '{\n "anchors": "task:RK-1"\n}' })).toEqual({
      name: 'rekall · rekall_context',
      target: '{',
      icon: 'plug'
    })
  })
})

describe('describePermission for the new tools', () => {
  it('says what Claude wants in a sentence', () => {
    expect(describePermission({ tool: 'todos', items: [] })).toBe('Claude wants to update its todo list')
    expect(describePermission({ tool: 'agent', description: 'Find it', subagentType: null, prompt: 'p' })).toBe(
      'Claude wants to start an agent: Find it'
    )
    expect(describePermission({ tool: 'web', name: 'WebFetch', target: 'https://a.io' })).toBe(
      'Claude wants to use WebFetch on https://a.io'
    )
    expect(describePermission({ tool: 'mcp', server: 'rekall', toolName: 'rekall_note', text: '' })).toBe(
      'Claude wants to use rekall_note of rekall'
    )
  })
})

describe('mcpArguments', () => {
  it('reads a JSON object as key and value lines, text as it is and objects compact', () => {
    expect(mcpArguments(JSON.stringify({ a: 'x', b: 2, c: { d: 1 } }))).toEqual([
      { key: 'a', value: 'x' },
      { key: 'b', value: '2' },
      { key: 'c', value: '{"d":1}' }
    ])
  })

  it('keeps text that is not an object as one line with no key, and has none for no text', () => {
    expect(mcpArguments('plain words')).toEqual([{ key: '', value: 'plain words' }])
    expect(mcpArguments('[1,2]')).toEqual([{ key: '', value: '[1,2]' }])
    expect(mcpArguments('')).toEqual([])
  })
})
