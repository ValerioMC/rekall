import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import HelpSection from '@/features/settings/components/HelpSection.vue'
import { HELP_CONTEXTS, HELP_TOOLS } from '@/features/settings/model/help-context'

function mountHelp() {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { render: () => null } }] })
  return mount(HelpSection, { global: { plugins: [router] } })
}

describe('the Help section', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('opens on Load and offers every context as a tab', () => {
    const help = mountHelp()

    expect(help.findAll('[role="tab"]').map((tab) => tab.text())).toEqual(HELP_CONTEXTS.map((context) => context.label))
    expect(help.get('[data-testid="sub-nav-load"]').attributes('aria-selected')).toBe('true')
    expect(help.get('[data-testid="help-commands"]').text()).toContain('/rk task:RK-12')
  })

  it('shows the commands and the tools of the context picked', async () => {
    const help = mountHelp()

    await help.get('[data-testid="sub-nav-steps"]').trigger('click')

    const commands = help.findAll('[data-testid="help-command"] code').map((line) => line.text())
    expect(commands).toContain('/rk task:RK-12 step:3 start')
    expect(commands).toContain('/rk task:RK-12 plan flow')
    expect(help.findAll('[data-testid="help-tool"] code').map((name) => name.text())).toContain('rekall_step')
    expect(help.find('[data-testid="help-examples"]').exists()).toBe(false)
  })

  it('marks which tools only read and which write', async () => {
    const help = mountHelp()

    await help.get('[data-testid="sub-nav-load"]').trigger('click')

    expect(help.findAll('[data-testid="help-tool-access-read"]')).toHaveLength(3)
    expect(help.find('[data-testid="help-tool-access-write"]').exists()).toBe(false)
  })

  it('shows requests to paste for a session outside Rekall, among them creating a task', async () => {
    const help = mountHelp()

    await help.get('[data-testid="sub-nav-outside"]').trigger('click')

    expect(help.find('[data-testid="help-commands"]').exists()).toBe(false)
    const prompts = help.findAll('[data-testid="help-example-prompt"]').map((prompt) => prompt.text())
    expect(prompts.some((prompt) => prompt.includes('create a task in project vega'))).toBe(true)
    expect(help.get('[data-testid="help-examples"]').text()).toContain('rekall_create_task')
  })

  it('copies the command that was clicked', async () => {
    const writeText = vi.fn(() => Promise.resolve())
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true })
    const help = mountHelp()

    await help.get('[data-testid="help-command"] [data-testid="help-copy"]').trigger('click')

    expect(writeText).toHaveBeenCalledWith('/rk task:RK-12')
    expect(help.get('[data-testid="help-copy"] [data-testid="copy-glyph"]').attributes('data-copied')).toBe('true')
  })
})

describe('the help catalog', () => {
  it('lists only tools it describes, once per context', () => {
    for (const context of HELP_CONTEXTS) {
      const names = [...context.tools, ...context.examples.flatMap((example) => example.tools)]
      expect(names.every((name) => name in HELP_TOOLS)).toBe(true)
      expect(new Set(context.tools).size).toBe(context.tools.length)
    }
  })
})
