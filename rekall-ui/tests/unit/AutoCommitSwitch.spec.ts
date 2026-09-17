import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import AutoCommitSwitch from '@/components/catalog/AutoCommitSwitch.vue'
import type { ProjectRepository } from '@/model/catalog'

/**
 * The switch only arms on a folder git recognises as a repository. Everything else it says
 * is why it will not arm, or whom a commit would be signed by once it does.
 */

function repository(over: Partial<ProjectRepository> = {}): ProjectRepository {
  return {
    folder: '/Users/someone/Projects/vega',
    exists: true,
    repository: true,
    branch: 'main',
    userName: 'Someone',
    userEmail: 'someone@example.com',
    autoCommit: false,
    ...over
  }
}

function mountSwitch(props: { modelValue: boolean; repository: ProjectRepository | null; pending?: boolean; saving?: boolean }) {
  return mount(AutoCommitSwitch, { props })
}

describe('AutoCommitSwitch', () => {
  it('arms on a repository and shows the branch and the identity a commit would carry', async () => {
    const wrapper = mountSwitch({ modelValue: false, repository: repository() })

    const control = wrapper.get('[data-testid="auto-commit-switch"]')
    expect(control.attributes('aria-disabled')).toBe('false')
    expect(control.attributes('aria-checked')).toBe('false')
    expect(wrapper.find('[data-testid="auto-commit-reason"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="auto-commit-repository"]').text())
      .toContain('main')
    expect(wrapper.get('[data-testid="auto-commit-repository"]').text())
      .toContain('Someone <someone@example.com>')

    await control.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[true]])
  })

  it('reads as on only while the folder is a repository', () => {
    const on = mountSwitch({ modelValue: true, repository: repository() })
    expect(on.get('[data-testid="auto-commit-switch"]').attributes('aria-checked')).toBe('true')
    expect(on.get('[data-testid="auto-commit"]').classes()).toContain('auto-commit-on')

    const stale = mountSwitch({ modelValue: true, repository: repository({ repository: false, branch: null }) })
    expect(stale.get('[data-testid="auto-commit-switch"]').attributes('aria-checked')).toBe('false')
  })

  it('will not arm on a plain folder, and says what to do about it', async () => {
    const wrapper = mountSwitch({
      modelValue: false,
      repository: repository({ repository: false, branch: null, userName: null, userEmail: null })
    })

    const control = wrapper.get('[data-testid="auto-commit-switch"]')
    expect(control.attributes('aria-disabled')).toBe('true')
    expect(wrapper.get('[data-testid="auto-commit"]').attributes('data-readiness')).toBe('not-repository')
    expect(wrapper.get('[data-testid="auto-commit-reason"]').text()).toContain('Not a git repository')
    expect(wrapper.find('[data-testid="auto-commit-repository"]').exists()).toBe(false)

    await control.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
  })

  it('names a missing folder and an unset one differently', () => {
    const missing = mountSwitch({ modelValue: false, repository: repository({ exists: false, repository: false }) })
    expect(missing.get('[data-testid="auto-commit-reason"]').text()).toContain('not on this machine')

    const unset = mountSwitch({ modelValue: false, repository: repository({ folder: null, exists: false, repository: false }) })
    expect(unset.get('[data-testid="auto-commit-reason"]').text()).toContain('Set the folder above first')

    const unread = mountSwitch({ modelValue: false, repository: null })
    expect(unread.get('[data-testid="auto-commit"]').attributes('data-readiness')).toBe('unset')
  })

  it('still arms on a repository with no git identity, but warns the commit will fail', async () => {
    const wrapper = mountSwitch({ modelValue: false, repository: repository({ userName: null, userEmail: null }) })

    expect(wrapper.get('[data-testid="auto-commit-switch"]').attributes('aria-disabled')).toBe('false')
    expect(wrapper.get('[data-testid="auto-commit-reason"]').text()).toContain('user.email')
    expect(wrapper.get('[data-testid="auto-commit-reason"]').classes()).toContain('text-warn')

    await wrapper.get('[data-testid="auto-commit-switch"]').trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[true]])
  })

  it('holds still while the status is being read or a toggle is being saved', async () => {
    const pending = mountSwitch({ modelValue: false, repository: repository(), pending: true })
    await pending.get('[data-testid="auto-commit-switch"]').trigger('click')
    expect(pending.emitted('update:modelValue')).toBeUndefined()

    const saving = mountSwitch({ modelValue: true, repository: repository(), saving: true })
    await saving.get('[data-testid="auto-commit-switch"]').trigger('click')
    expect(saving.emitted('update:modelValue')).toBeUndefined()
    expect(saving.get('[data-testid="auto-commit-switch"]').classes()).toContain('auto-commit-switch-busy')
  })
})
