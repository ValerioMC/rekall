import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ToolCallCard from '@/features/chat/components/ToolCallCard.vue'
import type { ToolItem } from '@/features/chat/model/timeline'
import { toolItem } from './support'

function mountCard(call: ToolItem) {
  return mount(ToolCallCard, { props: { call } })
}

const BASH = toolItem({ input: { tool: 'bash', command: 'cargo test --workspace', description: 'Run every test' } })

describe('ToolCallCard', () => {
  it('is one row until it is opened, and opens and closes on its button', async () => {
    const card = mountCard(BASH)

    expect(card.get('button').attributes('aria-expanded')).toBe('false')
    expect(card.find('[data-testid=tool-call-body]').exists()).toBe(false)
    expect(card.text()).toContain('Bash')
    expect(card.text()).toContain('cargo test --workspace')

    await card.get('button').trigger('click')
    expect(card.get('button').attributes('aria-expanded')).toBe('true')
    expect(card.get('[data-testid=tool-call-body]').text()).toContain('Run every test')

    await card.get('button').trigger('click')
    expect(card.find('[data-testid=tool-call-body]').exists()).toBe(false)
  })

  it('says what state the call is in', () => {
    expect(mountCard(BASH).get('button [data-state]').text()).toBe('Running')
    expect(mountCard({ ...BASH, state: 'done' }).text()).toContain('Done')
    expect(mountCard({ ...BASH, state: 'denied' }).text()).toContain('Denied')
  })

  it('shows the end of a command\'s output once opened', async () => {
    const card = mountCard({
      ...BASH,
      state: 'done',
      result: { isError: false, excerpt: 'test result: ok. 12 passed', truncated: false }
    })

    await card.get('button').trigger('click')

    expect(card.get('[data-testid=tool-output]').text()).toContain('12 passed')
  })

  it('shows an edit as the lines it removes and the lines it adds', async () => {
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
        result: { isError: false, excerpt: 'The file has been updated', truncated: false }
      })
    )

    await card.get('button').trigger('click')

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

  it('shows the output of an edit that failed', async () => {
    const card = mountCard(
      toolItem({
        input: { tool: 'edit', filePath: 'a.rs', oldString: 'x', newString: 'y', replaceAll: false },
        state: 'failed',
        result: { isError: true, excerpt: 'old_string not found', truncated: false }
      })
    )

    await card.get('button').trigger('click')

    expect(card.get('[data-testid=tool-output]').text()).toContain('old_string not found')
  })

  it('shows a file written as all added lines, the first 40 and then a key for the rest', async () => {
    const content = Array.from({ length: 45 }, (_, index) => `line ${index + 1}`).join('\n')
    const card = mountCard(toolItem({ input: { tool: 'write', filePath: 'out.txt', content } }))

    await card.get('button').trigger('click')
    expect(card.findAll('[data-sign]')).toHaveLength(40)

    const more = card.findAll('button').find((button) => button.text() === 'Show 5 more lines')
    await more!.trigger('click')
    expect(card.findAll('[data-sign]')).toHaveLength(45)
  })

  it('cuts a long output to its last 40 lines, with a key for all of them and a note when Rekall cut it', async () => {
    const excerpt = Array.from({ length: 100 }, (_, index) => `out ${index + 1}`).join('\n')
    const card = mountCard({ ...BASH, state: 'done', result: { isError: false, excerpt, truncated: true } })

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
})
