import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createConsoleState } from '@/stores/console/console-state'
import { useSkillAssignments } from '@/stores/console/useSkillAssignments'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'

const api = vi.hoisted(() => ({ setTaskSkills: vi.fn(), setStepSkills: vi.fn(), setTagSkills: vi.fn() }))
vi.mock('@/api/skill-assignments.api', () => api)

const task = 't1' as TaskId
const step = 's1' as TaskStepId
const tag = 'g1' as TagId

beforeEach(() => vi.resetAllMocks())

describe('useSkillAssignments', () => {
  it('replaces the task own skills and leaves its steps and other tasks alone', async () => {
    const state = createConsoleState()
    state.taskSkills.value = [
      { taskId: task, stepId: null, skillName: 'old' },
      { taskId: task, stepId: step, skillName: 'of-the-step' },
      { taskId: 't2' as TaskId, stepId: null, skillName: 'other-task' }
    ]
    api.setTaskSkills.mockResolvedValue([{ taskId: task, stepId: null, skillName: 'new' }])

    await useSkillAssignments(state).assignTaskSkills(task, ['new'])

    expect(api.setTaskSkills).toHaveBeenCalledWith(task, ['new'])
    expect(state.taskSkills.value.map((skill) => skill.skillName).sort()).toEqual(['new', 'of-the-step', 'other-task'])
  })

  it('replaces one step skills and nothing else', async () => {
    const state = createConsoleState()
    state.taskSkills.value = [
      { taskId: task, stepId: null, skillName: 'task-level' },
      { taskId: task, stepId: step, skillName: 'old' }
    ]
    api.setStepSkills.mockResolvedValue([])

    await useSkillAssignments(state).assignStepSkills(step, [])

    expect(state.taskSkills.value.map((skill) => skill.skillName)).toEqual(['task-level'])
  })

  it('replaces one tag skills', async () => {
    const state = createConsoleState()
    state.tagSkills.value = [
      { tagId: tag, skillName: 'old' },
      { tagId: 'g2' as TagId, skillName: 'kept' }
    ]
    api.setTagSkills.mockResolvedValue([{ tagId: tag, skillName: 'new' }])

    await useSkillAssignments(state).assignTagSkills(tag, ['new'])

    expect(state.tagSkills.value.map((skill) => skill.skillName).sort()).toEqual(['kept', 'new'])
  })

  it('keeps the rows as they were when the write fails', async () => {
    const state = createConsoleState()
    state.tagSkills.value = [{ tagId: tag, skillName: 'old' }]
    api.setTagSkills.mockRejectedValue(new Error('refused'))

    await expect(useSkillAssignments(state).assignTagSkills(tag, ['new'])).rejects.toThrow('refused')

    expect(state.tagSkills.value.map((skill) => skill.skillName)).toEqual(['old'])
  })
})
