import MarkdownIt from 'markdown-it'
import { describe, expect, it } from 'vitest'
import { markComponentReferences } from '@/common/markdown/component-reference'
import { paintTaskReferences } from '@/common/markdown/task-reference'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'
import { task } from './features/diagram/diagram.fixtures'

/** The page draws `[[task:label]]` as a chip and settles it against the project's tasks. */
function render(markdown: string): string {
  const md = new MarkdownIt({ html: false })
  markComponentReferences(md as never)
  return md.render(markdown)
}

const EXPORT: Task = { ...task('t-1' as TaskId, 'export', 'p-1' as ProjectId), title: 'Export the report' }

function chipOf(markdown: string, tasks: readonly Task[] | null): HTMLElement | null {
  const page = document.createElement('div')
  page.innerHTML = render(markdown)
  paintTaskReferences(page, tasks)
  return page.querySelector<HTMLElement>('.rk-task-ref')
}

describe('the [[task:label]] syntax', () => {
  it('becomes a task chip carrying the label, never a component chip', () => {
    const html = render('Build on [[task:export]].')

    expect(html).toContain('<span class="rk-task-ref" data-task="export" data-state="plain">export</span>')
    expect(html).not.toContain('rk-component-ref')
  })

  it('is found with its title and status on hover, ignoring case', () => {
    const chip = chipOf('[[task:EXPORT]]', [EXPORT])

    expect(chip?.dataset['state']).toBe('found')
    expect(chip?.title).toBe('Export the report\nIn progress')
  })

  it('is missing when the project has no such task, and plain with no project', () => {
    expect(chipOf('[[task:import]]', [EXPORT])?.dataset['state']).toBe('missing')
    expect(chipOf('[[task:export]]', null)?.dataset['state']).toBe('plain')
  })
})
