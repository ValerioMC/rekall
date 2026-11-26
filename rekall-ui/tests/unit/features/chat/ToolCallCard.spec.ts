import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ToolCallCard from '@/features/chat/components/ToolCallCard.vue'
import type { ToolItem } from '@/features/chat/model/timeline'
import { toolItem } from './support'

function mountCard(call: ToolItem) {
  return mount(ToolCallCard, { props: { call } })
}

const SEARCH = toolItem({ input: { tool: 'search', name: 'Grep', pattern: 'fn main', path: 'src' } })

describe('ToolCallCard', () => {
  it('is one row until it is opened, and opens and closes on its button', async () => {
    const card = mountCard(SEARCH)

    expect(card.get('button').attributes('aria-expanded')).toBe('false')
    expect(card.find('[data-testid=tool-call-body]').exists()).toBe(false)
    expect(card.text()).toContain('Grep')
    expect(card.text()).toContain('fn main in src')

    await card.get('button').trigger('click')
    expect(card.get('button').attributes('aria-expanded')).toBe('true')
    expect(card.find('[data-testid=tool-call-body]').exists()).toBe(true)

    await card.get('button').trigger('click')
    expect(card.find('[data-testid=tool-call-body]').exists()).toBe(false)
  })

  it('says what state the call is in', () => {
    expect(mountCard(SEARCH).get('button [data-state]').text()).toBe('Running')
    expect(mountCard({ ...SEARCH, state: 'done' }).text()).toContain('Done')
    expect(mountCard({ ...SEARCH, state: 'denied' }).text()).toContain('Denied')
  })

  it('shows the end of a call\'s output once opened', async () => {
    const card = mountCard({
      ...SEARCH,
      state: 'done',
      result: { isError: false, excerpt: 'test result: ok. 12 passed', truncated: false, summary: null }
    })

    await card.get('button').trigger('click')

    expect(card.get('[data-testid=tool-output]').text()).toContain('12 passed')
  })

  it('shows an edit open, as the lines it removes and the lines it adds', () => {
    const card = mountCard(
      toolItem({
        input: {
          tool: 'edit',
          filePath: 'src/lib.rs',
          oldString: 'let total = 1;\nlet next = 2;',
          newString: 'let total = 3;',
          replaceAll: true
        },
        state: 'done',
        result: { isError: false, excerpt: 'The file has been updated', truncated: false, summary: null }
      })
    )

    expect(card.get('button').attributes('aria-expanded')).toBe('true')
    const rows = card.findAll('[data-sign]')
    expect(rows.map((row) => row.attributes('data-sign'))).toEqual(['-', '-', '+'])
    expect(rows.map((row) => row.text())).toEqual([
      '−removedlet total = 1;',
      '−removedlet next = 2;',
      '+addedlet total = 3;'
    ])
    expect(card.text()).toContain('all occurrences')
    expect(card.find('[data-testid=tool-output]').exists()).toBe(false)
  })

  it('shows the output of an edit that failed', () => {
    const card = mountCard(
      toolItem({
        input: { tool: 'edit', filePath: 'a.rs', oldString: 'x', newString: 'y', replaceAll: false },
        state: 'failed',
        result: { isError: true, excerpt: 'old_string not found', truncated: false, summary: null }
      })
    )

    expect(card.get('[data-testid=tool-output]').text()).toContain('old_string not found')
  })

  it('shows a file written as all added lines, the first 40 and then a key for the rest', async () => {
    const content = Array.from({ length: 45 }, (_, index) => `line ${index + 1}`).join('\n')
    const card = mountCard(toolItem({ input: { tool: 'write', filePath: 'out.txt', content } }))

    expect(card.findAll('[data-sign]')).toHaveLength(40)

    const more = card.findAll('button').find((button) => button.text() === 'Show 5 more lines')
    await more!.trigger('click')
    expect(card.findAll('[data-sign]')).toHaveLength(45)
  })

  it('cuts a long output to its last 40 lines, with a key for all of them and a note when Rekall cut it', async () => {
    const excerpt = Array.from({ length: 100 }, (_, index) => `out ${index + 1}`).join('\n')
    const card = mountCard({ ...SEARCH, state: 'done', result: { isError: false, excerpt, truncated: true, summary: null } })

    await card.get('button').trigger('click')
    const output = card.get('[data-testid=tool-output]')
    expect(output.get('pre').text().split('\n')).toHaveLength(40)
    expect(output.get('pre').text().startsWith('out 61')).toBe(true)
    expect(output.get('pre').text().endsWith('out 100')).toBe(true)
    expect(output.text()).toContain('output cut by Rekall')

    await output.get('button').trigger('click')
    expect(output.get('pre').text().split('\n')).toHaveLength(100)
    expect(output.find('button').exists()).toBe(false)
  })

  it('shows the input of a tool it has no view for', async () => {
    const card = mountCard(toolItem({ input: { tool: 'other', name: 'WebFetch', text: 'https://example.com' } }))

    await card.get('button').trigger('click')

    expect(card.get('[data-testid=tool-call-body]').text()).toContain('https://example.com')
  })

  it('says what the call did after what it was pointed at, without opening it', () => {
    const card = mountCard({
      ...SEARCH,
      state: 'failed',
      result: { isError: true, excerpt: 'boom', truncated: false, summary: '3 matches' }
    })

    expect(card.get('[data-testid=tool-summary]').text()).toBe('3 matches')
    expect(card.find('[data-testid=tool-call-body]').exists()).toBe(false)
  })

  it('shows no summary when the result has none', () => {
    const card = mountCard({ ...SEARCH, state: 'done', result: { isError: false, excerpt: 'ok', truncated: false, summary: null } })

    expect(card.find('[data-testid=tool-summary]').exists()).toBe(false)
  })

  it('heads each file of a change set with its path, and shows no output when it has none to show', () => {
    const card = mountCard(
      toolItem({
        input: {
          tool: 'fileChanges',
          changes: [
            { path: 'notes.md', kind: 'add', movedTo: null, diff: 'hello\n' },
            { path: 'old.rs', kind: 'update', movedTo: 'new.rs', diff: '@@ -1,2 +1,2 @@\n keep\n-a\n+b\n' }
          ]
        },
        state: 'failed',
        result: { isError: true, excerpt: '', truncated: false, summary: null }
      })
    )

    const files = card.findAll('[data-testid=tool-file]')
    expect(card.text()).toContain('2 files')
    expect(files.map((file) => file.get('p').text())).toEqual(['notes.md', 'old.rsmoved to new.rs'])
    expect(files[1]!.findAll('[data-sign]').map((row) => row.attributes('data-sign'))).toEqual(['@', ' ', '-', '+'])
    expect(card.find('[data-testid=tool-output]').exists()).toBe(false)
  })

  it('names a single changed file once, whole in the row\'s title', () => {
    const card = mountCard(
      toolItem({ input: { tool: 'fileChanges', changes: [{ path: 'src/main.rs', kind: 'delete', movedTo: null, diff: 'gone' }] } })
    )

    expect(card.get('[title="src/main.rs"]').text()).toBe('src/main.rs')
    expect(card.get('[data-testid=tool-file]').find('p').exists()).toBe(false)
  })

  it('reads a todo list open, in progress first, a glyph for each status', () => {
    const card = mountCard(
      toolItem({
        input: {
          tool: 'todos',
          items: [
            { content: 'write tests', status: 'pending' },
            { content: 'build parser', status: 'completed' },
            { content: 'wire route', status: 'inProgress' }
          ]
        }
      })
    )

    expect(card.get('button').attributes('aria-expanded')).toBe('true')
    const rows = card.findAll('[data-testid=tool-todos] li')
    expect(rows.map((row) => row.text())).toEqual(['In progresswire route', 'Pendingwrite tests', 'Completedbuild parser'])
    expect(card.text()).toContain('1 of 3 done')
  })

  it('keeps every other call closed', () => {
    const web = mountCard(toolItem({ input: { tool: 'web', name: 'WebFetch', target: 'https://example.com/a' } }))
    const agent = mountCard(
      toolItem({ input: { tool: 'agent', description: 'Find it', subagentType: null, prompt: 'p' }, state: 'done' })
    )

    expect(web.get('button').attributes('aria-expanded')).toBe('false')
    expect(agent.get('button').attributes('aria-expanded')).toBe('false')
  })

  it('shows a web call as its host in the row and its whole address in the body', async () => {
    const card = mountCard(toolItem({ input: { tool: 'web', name: 'WebFetch', target: 'https://example.com/docs/a' } }))

    expect(card.text()).toContain('example.com')
    await card.get('button').trigger('click')

    expect(card.get('[data-testid=tool-web-target]').text()).toBe('https://example.com/docs/a')
  })

  it('names an mcp call by server and tool and reads its arguments as lines', async () => {
    const card = mountCard(
      toolItem({
        input: { tool: 'mcp', server: 'rekall', toolName: 'rekall_context', text: JSON.stringify({ anchors: 'task:RK-1', limit: 3 }, null, 2) }
      })
    )

    expect(card.text()).toContain('rekall · rekall_context')
    await card.get('button').trigger('click')

    const arguments_ = card.get('[data-testid=tool-arguments]')
    expect(arguments_.findAll('dt').map((key) => key.text())).toEqual(['anchors', 'limit'])
    expect(arguments_.findAll('dd').map((value) => value.text())).toEqual(['task:RK-1', '3'])
    expect(arguments_.text()).not.toContain('{')
  })

  it('opens an Agent while it runs and folds it when its result arrives, whatever it was before', async () => {
    const agent = toolItem({ input: { tool: 'agent', description: 'Find it', subagentType: null, prompt: 'p' } })
    const card = mountCard(agent)
    expect(card.get('button').attributes('aria-expanded')).toBe('true')

    await card.setProps({ call: { ...agent, state: 'done', result: { isError: false, excerpt: 'found', truncated: false, summary: null } } })

    expect(card.get('button').attributes('aria-expanded')).toBe('false')
  })

  it('counts the calls of a subagent while it is folded', async () => {
    const agent = toolItem({
      input: { tool: 'agent', description: 'Find it', subagentType: null, prompt: 'p' },
      state: 'done'
    })
    const card = mount(ToolCallCard, { props: { call: agent, innerCalls: 3 } })

    expect(card.get('[data-testid=tool-inner-count]').text()).toBe('3 calls')
    await card.get('button').trigger('click')
    expect(card.find('[data-testid=tool-inner-count]').exists()).toBe(false)
  })
})
