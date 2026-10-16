import { describe, expect, it } from 'vitest'
import {
  describeSkills,
  invocationName,
  skillsOfTag,
  stepSkills,
  taskSkills,
  type TagSkill,
  type TaskSkill
} from '@/features/skills/model/skill-assignment'
import { summary } from './claude-resources.fixtures'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'

const task = 't1' as TaskId
const other = 't2' as TaskId
const step = 's1' as TaskStepId
const tag = 'g1' as TagId

const owned: TaskSkill[] = [
  { taskId: task, stepId: null, skillName: 'rust-standards' },
  { taskId: task, stepId: step, skillName: 'ui-ux-pro-max' },
  { taskId: other, stepId: null, skillName: 'elsewhere' }
]
const tagged: TagSkill[] = [
  { tagId: tag, skillName: 'code-review' },
  { tagId: tag, skillName: 'rust-standards' }
]

describe('invocationName', () => {
  it('is a plugin skill qualified by its plugin, any other skill its own name', () => {
    const plugin = summary('plugins/p/skills/docx/SKILL.md', {
      kind: 'skill',
      name: 'docx',
      source: { kind: 'plugin', name: 'anthropic-skills', detail: null }
    })
    const own = summary('skills/rk/SKILL.md', { kind: 'skill', name: 'rk', source: { kind: 'own', name: 'You', detail: null } })

    expect(invocationName(plugin)).toBe('anthropic-skills:docx')
    expect(invocationName(own)).toBe('rk')
  })
})

describe('taskSkills', () => {
  it('lists the task own skills first, then the tag ones, a repeat only once', () => {
    expect(taskSkills(task, tag, owned, tagged)).toEqual([
      { name: 'rust-standards', origin: 'task' },
      { name: 'code-review', origin: 'tag' }
    ])
  })

  it('reads nothing from another task or from a task with no tag', () => {
    expect(taskSkills(other, null, owned, tagged)).toEqual([{ name: 'elsewhere', origin: 'task' }])
  })
})

describe('skillsOfTag', () => {
  it('is empty for no tag', () => {
    expect(skillsOfTag(null, tagged)).toEqual([])
  })
})

describe('stepSkills', () => {
  it('puts the step own skills before everything the task already carries', () => {
    const taskLevel = taskSkills(task, tag, owned, tagged)

    expect(stepSkills(step, taskLevel, owned).map((skill) => `${skill.origin}:${skill.name}`)).toEqual([
      'step:ui-ux-pro-max',
      'task:rust-standards',
      'tag:code-review'
    ])
  })

  it('does not list a step skill the task already has', () => {
    const taskLevel = taskSkills(task, null, owned, [])
    const duplicate: TaskSkill[] = [...owned, { taskId: task, stepId: step, skillName: 'rust-standards' }]

    expect(stepSkills(step, taskLevel, duplicate).filter((skill) => skill.name === 'rust-standards')).toHaveLength(1)
  })
})

describe('describeSkills', () => {
  it('names each skill and where it comes from, one per line', () => {
    expect(describeSkills([{ name: 'a', origin: 'tag' }, { name: 'b', origin: 'step' }])).toBe(
      'a, from its tag\nb, from this step'
    )
  })
})
