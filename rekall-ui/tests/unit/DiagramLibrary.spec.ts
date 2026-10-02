import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DiagramLibrary from '@/components/diagram/DiagramLibrary.vue'
import { diagramsByTask } from '@/common/diagram/task-diagrams'
import type { DiagramSummary } from '@/model/diagram'
import type { Project, Task } from '@/model/catalog'
import type { DiagramId, ProjectId, TaskId, TerminalId } from '@/model/branded'
import type { PendingGeneration } from '@/stores/diagram.store'

const PROJECT = 'p-1' as ProjectId

const project = { id: PROJECT, title: 'Vega', icon: 'folder' } as unknown as Project

function task(id: string, title: string): Task {
  return { id: id as TaskId, label: id, title, status: 'IN_PROGRESS', projectId: PROJECT } as unknown as Task
}

function diagram(id: string, taskId: TaskId | null): DiagramSummary {
  return {
    id: id as DiagramId,
    projectId: PROJECT,
    taskId,
    title: `Diagram ${id}`,
    question: '',
    nodeCount: 5,
    edgeCount: 4,
    createdAt: '2026-09-01T10:00:00Z',
    updatedAt: '2026-09-01T10:00:00Z'
  }
}

const drawn = task('drawn', 'Has a diagram')
const drawing = task('drawing', 'Being drawn')
const bare = task('bare', 'No diagram')
const summaries = [diagram('d-1', drawn.id), diagram('d-loose', null)]
const generation: PendingGeneration = {
  key: 1,
  projectId: PROJECT,
  taskId: drawing.id,
  request: 'how it decides',
  terminalId: 'term-1' as TerminalId,
  startedAt: 0
}

function library(scope?: 'diagrams') {
  const wrapper = mount(DiagramLibrary, {
    props: {
      summaries,
      byTask: diagramsByTask(summaries),
      generating: new Set([drawing.id]),
      projects: [project],
      tasks: [bare, drawing, drawn],
      pending: [generation],
      selectedId: null,
      arrivedId: null,
      loaded: true
    }
  })
  if (scope) void wrapper.get(`[data-testid="diagram-scope-${scope}"]`).trigger('click')
  return wrapper
}

describe('DiagramLibrary', () => {
  it('tags a task being drawn and one already drawn, and leaves the bare one untagged', () => {
    const wrapper = library()
    expect(wrapper.findAll('[data-testid="diagram-tag-generating"]')).toHaveLength(1)
    expect(wrapper.findAll('[data-testid="diagram-tag-ready"]')).toHaveLength(1)
    expect(wrapper.findAll('[data-testid="diagram-task-bare"]')).toHaveLength(1)
  })

  it('lists generating first, then drawn, then bare tasks', () => {
    const titles = library().findAll('[data-testid^="diagram-task"]').map((row) => row.text())
    expect(titles[0]).toContain('Being drawn')
    expect(titles[1]).toContain('Has a diagram')
    expect(titles[2]).toContain('No diagram')
  })

  it('opens the latest diagram when its task is picked', async () => {
    const wrapper = library()
    const rows = wrapper.findAll('[data-testid="diagram-task"]')
    await rows[1]!.trigger('click')
    expect(wrapper.emitted('select')?.[0]).toEqual(['d-1'])
  })

  it('asks to generate for a task with no diagram', async () => {
    const wrapper = library()
    await wrapper.get('[data-testid="diagram-task-bare"]').trigger('click')
    expect(wrapper.emitted('generate')?.[0]).toEqual([bare])
  })

  it('goes to the session when a generating task is picked', async () => {
    const wrapper = library()
    await wrapper.findAll('[data-testid="diagram-task"]')[0]!.trigger('click')
    expect(wrapper.emitted('openTerminal')?.[0]).toEqual([generation])
  })

  it('shows only tasks with a diagram under the diagrams scope', async () => {
    const wrapper = library('diagrams')
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('[data-testid="diagram-task-bare"]')).toHaveLength(0)
  })
})
