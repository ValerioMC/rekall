import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import TaskDependencyPicker from '@/features/dependency/components/TaskDependencyPicker.vue'
import { taskFixture } from '../../support/taskFixture'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

const tasks: Task[] = [
  { ...taskFixture('archive' as TaskId), title: 'Archive', status: 'TODO' },
  { ...taskFixture('report' as TaskId), title: 'Report builder', status: 'IN_PROGRESS' }
]

function mountPicker(candidates: readonly Task[] = tasks) {
  const anchor = document.createElement('button')
  document.body.appendChild(anchor)
  return mount(TaskDependencyPicker, { props: { tasks: candidates, anchor }, attachTo: document.body })
}

const rows = (): HTMLButtonElement[] =>
  Array.from(document.body.querySelectorAll<HTMLButtonElement>('[data-testid="task-dependency-picker-row"]'))
const filterField = (): HTMLInputElement =>
  document.body.querySelector<HTMLInputElement>('[data-testid="task-dependency-picker-filter"]')!
const press = (key: string): void =>
  void window.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }))

async function typeIntoFilter(text: string): Promise<void> {
  filterField().value = text
  filterField().dispatchEvent(new Event('input'))
  await Promise.resolve()
}

describe('TaskDependencyPicker', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  it('lists each task with where it stands', () => {
    mountPicker()

    expect(rows().map((row) => row.textContent)).toEqual([
      expect.stringContaining('Archive'),
      expect.stringContaining('Report builder')
    ])
    expect(rows()[0]?.textContent).toContain('To do')
    expect(rows()[1]?.textContent).toContain('In progress')
  })

  it('picks the task of the row that was clicked', () => {
    const wrapper = mountPicker()

    rows()[1]?.click()

    expect(wrapper.emitted('pick')).toEqual([['report']])
  })

  it('narrows the list as the person types', async () => {
    mountPicker()

    await typeIntoFilter('REPO')

    expect(rows().map((row) => row.textContent)).toEqual([expect.stringContaining('Report builder')])
  })

  it('says so when nothing matches', async () => {
    mountPicker()

    await typeIntoFilter('zzz')

    expect(rows()).toHaveLength(0)
    expect(document.body.querySelector('[data-testid="task-dependency-picker-empty"]')?.textContent).toContain(
      'No task matches that'
    )
  })

  it('picks the highlighted task with the keyboard', async () => {
    const wrapper = mountPicker()

    press('ArrowDown')
    press('Enter')

    expect(wrapper.emitted('pick')).toEqual([['report']])
  })

  it('closes on Escape without picking', () => {
    const wrapper = mountPicker()

    press('Escape')

    expect(wrapper.emitted('close')).toHaveLength(1)
    expect(wrapper.emitted('pick')).toBeUndefined()
  })

  it('says there is nothing to depend on when the project has no other task', () => {
    mountPicker([])

    expect(document.body.querySelector('[data-testid="task-dependency-picker-empty"]')?.textContent).toContain(
      'No other task'
    )
  })
})
