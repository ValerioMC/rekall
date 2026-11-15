import { describe, expect, it } from 'vitest'
import {
  assignmentsOf,
  describeSkills,
  invocationKey,
  invocationName,
  skillsOfTag,
  stepSkills,
  taskSkills,
  type TagSkill,
  type TaskSkill
} from '@/features/skills/model/skill-assignment'
import { summary } from '../claude-resources/claude-resources.fixtures'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'

const task = 't1' as TaskId
const other = 't2' as TaskId
const step = 's1' as TaskStepId
const tag = 'g1' as TagId

const owned: TaskSkill[] = [
  { taskId: task, stepId: null, skillName: 'rust-standards', arguments: null },
  { taskId: task, stepId: step, skillName: 'ui-ux-pro-max', arguments: null },
  { taskId: other, stepId: null, skillName: 'elsewhere', arguments: null }
]
const tagged: TagSkill[] = [
  { tagId: tag, skillName: 'code-review', arguments: null },
  { tagId: tag, skillName: 'rust-standards', arguments: null }
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
      { name: 'rust-standards', arguments: null, origin: 'task' },
      { name: 'code-review', arguments: null, origin: 'tag' }
    ])
  })

  it('reads nothing from another task or from a task with no tag', () => {
    expect(taskSkills(other, null, owned, tagged)).toEqual([{ name: 'elsewhere', arguments: null, origin: 'task' }])
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
    const duplicate: TaskSkill[] = [...owned, { taskId: task, stepId: step, skillName: 'rust-standards', arguments: null }]

    expect(stepSkills(step, taskLevel, duplicate).filter((skill) => skill.name === 'rust-standards')).toHaveLength(1)
  })
})

describe('describeSkills', () => {
  it('names each skill and where it comes from, one per line', () => {
    expect(describeSkills([{ name: 'a', arguments: null, origin: 'tag' }, { name: 'b', arguments: null, origin: 'step' }])).toBe(
      'a, from its tag\nb, from this step'
    )
  })

  it('puts a skill arguments beside its name', () => {
    expect(describeSkills([{ name: 'impeccable', arguments: 'redesign website ui', origin: 'task' }])).toBe(
      'impeccable (redesign website ui), from this task'
    )
  })
})

describe('arguments', () => {
  it('carry through from the task rows and the tag rows', () => {
    const withArguments: TaskSkill[] = [{ taskId: task, stepId: null, skillName: 'impeccable', arguments: 'redesign' }]
    const fromTag: TagSkill[] = [{ tagId: tag, skillName: 'impeccable', arguments: 'polish' }]

    expect(taskSkills(task, tag, withArguments, fromTag)).toEqual([
      { name: 'impeccable', arguments: 'redesign', origin: 'task' },
      { name: 'impeccable', arguments: 'polish', origin: 'tag' }
    ])
  })

  it('make the same skill with the same arguments one invocation, listed once', () => {
    const withArguments: TaskSkill[] = [{ taskId: task, stepId: null, skillName: 'impeccable', arguments: 'redesign' }]
    const fromTag: TagSkill[] = [{ tagId: tag, skillName: 'impeccable', arguments: 'redesign' }]

    expect(taskSkills(task, tag, withArguments, fromTag)).toEqual([{ name: 'impeccable', arguments: 'redesign', origin: 'task' }])
  })

  it('list a step skill again when its arguments differ from the task one', () => {
    const rows: TaskSkill[] = [
      { taskId: task, stepId: null, skillName: 'impeccable', arguments: 'redesign' },
      { taskId: task, stepId: step, skillName: 'impeccable', arguments: 'polish' }
    ]
    const taskLevel = taskSkills(task, null, rows, [])

    expect(stepSkills(step, taskLevel, rows).map((skill) => `${skill.origin}:${skill.arguments}`)).toEqual(['step:polish', 'task:redesign'])
  })

  it('leave out a step skill identical in name and arguments to the task one', () => {
    const rows: TaskSkill[] = [
      { taskId: task, stepId: null, skillName: 'impeccable', arguments: 'redesign' },
      { taskId: task, stepId: step, skillName: 'impeccable', arguments: 'redesign' }
    ]
    const taskLevel = taskSkills(task, null, rows, [])

    expect(stepSkills(step, taskLevel, rows)).toHaveLength(1)
  })
})

describe('invocationKey', () => {
  it('differs when only the arguments differ', () => {
    expect(invocationKey({ name: 'a', arguments: 'x' })).not.toBe(invocationKey({ name: 'a', arguments: null }))
  })
})

describe('assignmentsOf', () => {
  it('keeps the arguments of a name already held and gives a new one none', () => {
    const current = [{ name: 'impeccable', arguments: 'redesign' }]

    expect(assignmentsOf(['impeccable', 'rk'], current)).toEqual([
      { name: 'impeccable', arguments: 'redesign' },
      { name: 'rk', arguments: null }
    ])
  })

  it('drops a name that is no longer picked', () => {
    expect(assignmentsOf(['rk'], [{ name: 'impeccable', arguments: 'redesign' }])).toEqual([{ name: 'rk', arguments: null }])
  })
})
