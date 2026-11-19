import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createConsoleState } from '@/stores/console/console-state'
import { useSkillAssignments } from '@/features/skills/store/useSkillAssignments'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'

const api = vi.hoisted(() => ({ setTaskSkills: vi.fn(), setStepSkills: vi.fn(), setTagSkills: vi.fn() }))
vi.mock('@/features/skills/api/skill-assignments.api', () => api)

const task = 't1' as TaskId
const step = 's1' as TaskStepId
const tag = 'g1' as TagId

beforeEach(() => vi.resetAllMocks())

describe('useSkillAssignments', () => {
  it('replaces the task own skills and leaves its steps and other tasks alone', async () => {
    const state = createConsoleState()
    state.taskSkills.value = [
      { taskId: task, stepId: null, skillName: 'old', arguments: null },
      { taskId: task, stepId: step, skillName: 'of-the-step', arguments: null },
      { taskId: 't2' as TaskId, stepId: null, skillName: 'other-task', arguments: null }
    ]
    api.setTaskSkills.mockResolvedValue([{ taskId: task, stepId: null, skillName: 'new', arguments: null }])

    await useSkillAssignments(state).assignTaskSkills(task, [{ name: 'new', arguments: null }])

    expect(api.setTaskSkills).toHaveBeenCalledWith(task, [{ name: 'new', arguments: null }])
    expect(state.taskSkills.value.map((skill) => skill.skillName).sort()).toEqual([
      'new',
      'of-the-step',
      'other-task'
    ])
  })

  it('replaces one step skills and nothing else', async () => {
    const state = createConsoleState()
    state.taskSkills.value = [
      { taskId: task, stepId: null, skillName: 'task-level', arguments: null },
      { taskId: task, stepId: step, skillName: 'old', arguments: null }
    ]
    api.setStepSkills.mockResolvedValue([])

    await useSkillAssignments(state).assignStepSkills(step, [])

    expect(state.taskSkills.value.map((skill) => skill.skillName)).toEqual(['task-level'])
  })

  it('replaces one tag skills', async () => {
    const state = createConsoleState()
    state.tagSkills.value = [
      { tagId: tag, skillName: 'old', arguments: null },
      { tagId: 'g2' as TagId, skillName: 'kept', arguments: null }
    ]
    api.setTagSkills.mockResolvedValue([{ tagId: tag, skillName: 'new', arguments: null }])

    await useSkillAssignments(state).assignTagSkills(tag, [{ name: 'new', arguments: null }])

    expect(state.tagSkills.value.map((skill) => skill.skillName).sort()).toEqual(['kept', 'new'])
  })

  it('keeps the rows as they were when the write fails', async () => {
    const state = createConsoleState()
    state.tagSkills.value = [{ tagId: tag, skillName: 'old', arguments: null }]
    api.setTagSkills.mockRejectedValue(new Error('refused'))

    await expect(
      useSkillAssignments(state).assignTagSkills(tag, [{ name: 'new', arguments: null }])
    ).rejects.toThrow('refused')

    expect(state.tagSkills.value.map((skill) => skill.skillName)).toEqual(['old'])
  })
})

describe('useSkillAssignments, arguments', () => {
  it('sends the arguments with the name and keeps the ones the server answers', async () => {
    const state = createConsoleState()
    const skills = [{ name: 'impeccable', arguments: 'redesign website ui' }]
    api.setTaskSkills.mockResolvedValue([
      { taskId: task, stepId: null, skillName: 'impeccable', arguments: 'redesign website ui' }
    ])

    await useSkillAssignments(state).assignTaskSkills(task, skills)

    expect(api.setTaskSkills).toHaveBeenCalledWith(task, skills)
    expect(state.taskSkills.value.map((skill) => skill.arguments)).toEqual(['redesign website ui'])
  })

  it('replaces the arguments of a step skill with the list it sends', async () => {
    const state = createConsoleState()
    state.taskSkills.value = [{ taskId: task, stepId: step, skillName: 'impeccable', arguments: 'redesign' }]
    api.setStepSkills.mockResolvedValue([
      { taskId: task, stepId: step, skillName: 'impeccable', arguments: 'polish' }
    ])

    await useSkillAssignments(state).assignStepSkills(step, [{ name: 'impeccable', arguments: 'polish' }])

    expect(state.taskSkills.value.map((skill) => skill.arguments)).toEqual(['polish'])
  })
})
