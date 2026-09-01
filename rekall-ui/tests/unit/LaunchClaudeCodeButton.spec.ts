import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import LaunchClaudeCodeButton from '@/components/claude/LaunchClaudeCodeButton.vue'
import { useToastStore } from '@/stores/toast.store'

/**
 * The button that opens a session. Everything it can get wrong is silent: a browser tab where it
 * should not exist at all, a project with no folder to open, a bridge that refuses. So each of
 * those is a test, and the payload is one too, because the native side accepts nothing else.
 */
const openInClaudeCode = vi.fn()

const ANCHORS = 'project:stvv task:env-vars-cv'
const FOLDER = '/Users/someone/Projects/stvv'

function installBridge(): void {
  window.rekallDesktop = { pickFolder: vi.fn(), openInClaudeCode }
}

/**
 * A storage of the test's own. The switch is read from the browser, and this environment does
 * not hand out a usable one, which is also the case the reader is written to survive.
 */
function installStorage(): Map<string, string> {
  const values = new Map<string, string>()
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => void values.set(key, value),
      removeItem: (key: string) => void values.delete(key),
      clear: () => values.clear()
    }
  })
  return values
}

function mountButton(folder: string | null = FOLDER) {
  return mount(LaunchClaudeCodeButton, { props: { anchors: ANCHORS, folder } })
}

describe('LaunchClaudeCodeButton', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    openInClaudeCode.mockReset()
    openInClaudeCode.mockResolvedValue('iTerm')
    installBridge()
    installStorage()
  })

  afterEach(() => {
    delete window.rekallDesktop
  })

  it('is not there at all in a browser, where no terminal can be opened', () => {
    delete window.rekallDesktop

    expect(mountButton().find('[data-testid="launch-claude-code"]').exists()).toBe(false)
  })

  it('sends the folder, the anchors and the permission answer, and names the terminal', async () => {
    const wrapper = mountButton()

    await wrapper.get('[data-testid="launch-claude-code"]').trigger('click')
    await flushPromises()

    expect(openInClaudeCode).toHaveBeenCalledWith({
      directory: FOLDER,
      anchors: ANCHORS,
      skipPermissions: false
    })
    expect(useToastStore().toasts[0]?.message).toBe('Opened in iTerm.')
  })

  it('carries the skip-permissions switch through when it is on', async () => {
    window.localStorage.setItem('rekall.claude.skip-permissions', 'true')
    const wrapper = mountButton()

    await wrapper.get('[data-testid="launch-claude-code"]').trigger('click')
    await flushPromises()

    expect(openInClaudeCode).toHaveBeenCalledWith(
      expect.objectContaining({ skipPermissions: true })
    )
  })

  /** Pressable rather than disabled: a disabled button carries no tooltip in this window, so
   *  the person who has not set a folder would be left with a button that looks broken. */
  it('says what is missing when the project has no folder, and opens nothing', async () => {
    const wrapper = mountButton(null)

    const button = wrapper.get('[data-testid="launch-claude-code"]')
    expect(button.attributes('disabled')).toBeUndefined()

    await button.trigger('click')
    await flushPromises()

    expect(openInClaudeCode).not.toHaveBeenCalled()
    expect(useToastStore().toasts[0]?.message).toContain('folder')
  })

  it('says why nothing opened when the bridge refuses', async () => {
    openInClaudeCode.mockRejectedValue(new Error('That folder is not there any more'))
    const wrapper = mountButton()

    await wrapper.get('[data-testid="launch-claude-code"]').trigger('click')
    await flushPromises()

    const toast = useToastStore().toasts[0]
    expect(toast?.kind).toBe('error')
    expect(toast?.message).toBe('That folder is not there any more')
  })
})
