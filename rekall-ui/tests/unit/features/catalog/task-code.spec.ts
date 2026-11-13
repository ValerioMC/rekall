import { describe, expect, it } from 'vitest'
import { codeNumber, namesTask, newestFirst, taskCodeOf } from '@/features/catalog/model/task-code'
import { taskDraft } from '@/features/catalog/model/record-draft'
import type { Project, Task } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

const vega = 'p1' as ProjectId
const beacon = 'p2' as ProjectId
const projects = [{ id: vega, taskKey: 'VG' }, { id: beacon, taskKey: 'BC' }] as Project[]
const task = (id: string, projectId: ProjectId, code: string): Task => ({ id: id as TaskId, projectId, code, label: 'x' }) as Task

describe('task codes', () => {
  it('reads the number a code ends with', () => {
    expect(codeNumber('RK-12')).toBe(12)
    expect(codeNumber('RK-x')).toBe(0)
    expect(codeNumber('')).toBe(0)
  })

  it('orders the higher number first, so 10 comes before 9', () => {
    const codes = [task('a', vega, 'VG-9'), task('b', vega, 'VG-10'), task('c', vega, 'VG-2')]

    expect([...codes].sort(newestFirst).map((t) => t.code)).toEqual(['VG-10', 'VG-9', 'VG-2'])
  })

  it('gives a new task the next number of its project and the key of that project', () => {
    const tasks = [task('a', vega, 'VG-4'), task('b', vega, 'VG-2'), task('c', beacon, 'BC-7')]

    expect(taskCodeOf(taskDraft(vega) as never, projects, tasks)).toBe('VG-5')
    expect(taskCodeOf(taskDraft(beacon) as never, projects, tasks)).toBe('BC-8')
  })

  it('keeps the code a task has, and gives the next one of the project it is moved to', () => {
    const stored = task('a', vega, 'VG-4')
    const tasks = [stored, task('c', beacon, 'BC-7')]
    const draft = taskDraft(vega, { ...stored, title: 'T', description: null, status: 'TODO', kind: 'TASK', epicId: null, tagId: null } as Task)

    expect(taskCodeOf(draft as never, projects, tasks)).toBe('VG-4')
    expect(taskCodeOf({ ...draft, projectId: beacon } as never, projects, tasks)).toBe('BC-8')
  })

  it('names a task by its code, or by the label an older reference still carries', () => {
    const found = { code: 'VG-4', label: 'report-builder' }

    expect(namesTask(found, ' vg-4 ')).toBe(true)
    expect(namesTask(found, 'Report-Builder')).toBe(true)
    expect(namesTask(found, 'VG-5')).toBe(false)
  })
})
