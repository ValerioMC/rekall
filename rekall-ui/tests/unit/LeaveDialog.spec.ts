import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import LeaveDialog from '@/components/shell/LeaveDialog.vue'

const QUESTION = {
  title: 'Quit Rekall?',
  message: '2 Claude sessions are still running.',
  confirm: 'Quit',
  stay: 'Keep working'
}

function ask(): void {
  window.dispatchEvent(new CustomEvent('rekall:leave', { detail: QUESTION }))
}

function installBridge() {
  const answerLeave = vi.fn(() => Promise.resolve())
  window.rekallDesktop = { pickFolder: vi.fn(), answerLeave }
  return answerLeave
}

let wrapper: ReturnType<typeof mount> | undefined

afterEach(() => {
  wrapper?.unmount()
  wrapper = undefined
  delete window.rekallDesktop
})

describe('LeaveDialog', () => {
  it('tells the shell it is listening, and stops when unmounted', () => {
    wrapper = mount(LeaveDialog)
    expect(window.rekallLeaveReady).toBe(true)

    wrapper.unmount()
    wrapper = undefined
    expect(window.rekallLeaveReady).toBe(false)
  })

  it('shows what the shell asked and nothing before it asks', async () => {
    wrapper = mount(LeaveDialog, { attachTo: document.body })
    expect(wrapper.find('[data-testid="leave-dialog"]').exists()).toBe(false)

    ask()
    await flushPromises()

    const dialog = wrapper.get('[data-testid="leave-dialog"]')
    expect(dialog.text()).toContain('Quit Rekall?')
    expect(dialog.text()).toContain('2 Claude sessions are still running.')
    expect(wrapper.get('[data-testid="leave-confirm"]').text()).toBe('Quit')
    expect(wrapper.get('[data-testid="leave-stay"]').text()).toBe('Keep working')
  })

  it('answers yes when the user confirms', async () => {
    const answerLeave = installBridge()
    wrapper = mount(LeaveDialog, { attachTo: document.body })
    ask()
    await flushPromises()

    await wrapper.get('[data-testid="leave-confirm"]').trigger('click')

    expect(answerLeave).toHaveBeenCalledWith(true)
    expect(wrapper.find('[data-testid="leave-dialog"]').exists()).toBe(false)
  })

  it('answers no on the stay button and on Escape', async () => {
    const answerLeave = installBridge()
    wrapper = mount(LeaveDialog, { attachTo: document.body })

    ask()
    await flushPromises()
    await wrapper.get('[data-testid="leave-stay"]').trigger('click')
    ask()
    await flushPromises()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()

    expect(answerLeave).toHaveBeenNthCalledWith(1, false)
    expect(answerLeave).toHaveBeenNthCalledWith(2, false)
  })
})
