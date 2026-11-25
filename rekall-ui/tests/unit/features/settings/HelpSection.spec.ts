import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import HelpSection from '@/features/settings/components/HelpSection.vue'
import { HELP_CONTEXTS, HELP_TOOLS } from '@/features/settings/model/help-context'

function mountHelp(contextId?: string) {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { render: () => null } }] })
  return mount(HelpSection, { props: { contextId }, global: { plugins: [router] } })
}

describe('the Help section', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('opens on Load when the address names no topic or an unknown one', () => {
    expect(mountHelp().get('[data-testid="help-panel"]').attributes('aria-label')).toBe('Load')
    expect(mountHelp('nowhere').get('[data-testid="help-panel"]').attributes('aria-label')).toBe('Load')
  })

  it('shows the commands, what each calls, and the tools asked directly for the topic', () => {
    const help = mountHelp('steps')

    const commands = help.findAll('[data-testid="help-command"] code').map((line) => line.text())
    expect(commands).toContain('/rk task:RK-12 step:3 start')
    expect(commands).toContain('/rk task:RK-12 plan flow')
    expect(commands).toContain('/rk task:RK-12 discuss')
    expect(help.get('[data-testid="help-command"]:nth-child(4) [data-testid="help-calls"]').text()).toContain('rekall_edit_step')
    expect(help.get('[data-testid="help-command"]:nth-child(5) [data-testid="help-calls"]').text()).toContain('rekall_step')
    expect(help.get('[data-testid="help-tools"]').text()).toContain('rekall_delete_step')
    expect(help.find('[data-testid="help-examples"]').exists()).toBe(false)
  })

  it('marks which tools only read and which write', () => {
    const help = mountHelp('load')

    expect(help.findAll('[data-testid="help-tool-access-read"]')).toHaveLength(2)
    expect(help.find('[data-testid="help-tool-access-write"]').exists()).toBe(false)
  })

  it('shows requests to paste for a session outside Rekall, among them creating a task', () => {
    const help = mountHelp('outside')

    expect(help.get('[data-testid="help-commands"]').text()).toContain('claude mcp add --scope user')
    const prompts = help.findAll('[data-testid="help-example-prompt"]').map((prompt) => prompt.text())
    expect(prompts.some((prompt) => prompt.includes('create a task in project vega'))).toBe(true)
  })

  it('puts the typed project and task into every command and request', async () => {
    const help = mountHelp('outside')
    const [project, task] = help.findAll('input')

    await project!.setValue('lightcraft')
    await task!.setValue('LC-7')

    const prompts = help.findAll('[data-testid="help-example-prompt"]').map((prompt) => prompt.text())
    expect(prompts.some((prompt) => prompt.includes('project lightcraft') && prompt.includes('task LC-7'))).toBe(true)
    expect(help.find('[data-testid="help-placeholder"]').exists()).toBe(false)
  })

  it('finds across every context and says when nothing matches', async () => {
    const help = mountHelp('load')
    const find = help.get('input[type="search"]')

    await find.setValue('create task')
    expect(help.findAll('[data-testid="help-panel"]').map((panel) => panel.attributes('aria-label'))).toEqual(['Tasks', 'Outside Rekall'])

    await find.setValue('zzzz')
    expect(help.get('[data-testid="empty-state"]').text()).toContain('Nothing matches')
  })

  it('copies the command with the typed values', async () => {
    const writeText = vi.fn(() => Promise.resolve())
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true })
    const help = mountHelp('load')
    await help.findAll('input')[1]!.setValue('LC-7')

    await help.get('[data-testid="help-command"] [data-testid="help-copy"]').trigger('click')

    expect(writeText).toHaveBeenCalledWith('/rk task:LC-7')
  })
})

describe('the help catalog', () => {
  it('names only tools it describes, once per context', () => {
    for (const context of HELP_CONTEXTS) {
      const names = [...context.directTools, ...context.commands.flatMap((command) => command.calls), ...context.examples.flatMap((example) => example.tools)]
      expect(names.every((name) => name in HELP_TOOLS)).toBe(true)
      expect(new Set(context.directTools).size).toBe(context.directTools.length)
    }
  })
})
