import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import QueueSettingsAside from '@/features/queue/components/QueueSettingsAside.vue'

function aside(parallelSessions: string) {
  return mount(QueueSettingsAside, {
    props: {
      now: 0,
      startIso: null,
      startInPast: false,
      when: 'now',
      startLocal: '',
      ceilingOn: false,
      ceiling: 85,
      model: 'default',
      effort: 'default',
      skipPermissions: false,
      sessionKind: 'TERMINAL',
      parallelSessions
    }
  })
}

describe('the Parallel sessions setting of the run queue', () => {
  it('offers one to four tasks at once', () => {
    const options = aside('1').get('#queue-parallel').findAll('option').map((option) => option.text())

    expect(options).toEqual(['One at a time', '2 at once', '3 at once', '4 at once'])
  })

  it('says the queue works in the project folder when it runs one task at a time', () => {
    expect(aside('1').get('[data-testid="run-queue-parallel-help"]').text()).toBe("Tasks run in turn, in the project's folder.")
  })

  it('says each task gets a worktree, that dependent tasks never overlap and that nothing is merged', () => {
    const help = aside('2').get('[data-testid="run-queue-parallel-help"]').text()

    expect(help).toContain('git worktree of its own')
    expect(help).toContain('depend on each other never run together')
    expect(help).toContain('Nothing is merged for you.')
  })

  it('reaches the draft when changed', async () => {
    const wrapper = aside('1')

    await wrapper.get('#queue-parallel').setValue('3')

    expect(wrapper.emitted('update:parallelSessions')).toEqual([['3']])
  })
})
