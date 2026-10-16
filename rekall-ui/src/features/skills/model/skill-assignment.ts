import type { ResourceSummary } from '@/features/claude-resources/model/claude-resources'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'

/** A skill on a task (`stepId` null) or on one of its steps. */
export interface TaskSkill {
  readonly taskId: TaskId
  readonly stepId: TaskStepId | null
  readonly skillName: string
}

/** A skill every task carrying the tag uses by default. */
export interface TagSkill {
  readonly tagId: TagId
  readonly skillName: string
}

/** Where a skill on a task or step comes from, which decides whether it can be taken off there. */
export type SkillOrigin = 'step' | 'task' | 'tag'

export interface EffectiveSkill {
  readonly name: string
  readonly origin: SkillOrigin
}

export const SKILLS_PER_OWNER_MAX = 20

/** What a session types after the slash: a plugin's skill is `plugin:skill`, any other its own name. */
export function invocationName(skill: ResourceSummary): string {
  return skill.source?.kind === 'plugin' ? `${skill.source.name}:${skill.name}` : skill.name
}

/** The skills a tag brings to every task under it. */
export function skillsOfTag(tagId: TagId | null, tagged: readonly TagSkill[]): readonly EffectiveSkill[] {
  if (tagId === null) return []
  return tagged
    .filter((skill) => skill.tagId === tagId)
    .map((skill): EffectiveSkill => ({ name: skill.skillName, origin: 'tag' }))
}

/** The task's own skills, then its tag's; a skill the task holds itself is not listed twice. */
export function taskSkills(
  taskId: TaskId,
  tagId: TagId | null,
  owned: readonly TaskSkill[],
  tagged: readonly TagSkill[]
): readonly EffectiveSkill[] {
  const own = owned.filter((skill) => skill.taskId === taskId && skill.stepId === null)
  const names = new Set(own.map((skill) => skill.skillName))
  return [
    ...own.map((skill): EffectiveSkill => ({ name: skill.skillName, origin: 'task' })),
    ...skillsOfTag(tagId, tagged).filter((skill) => !names.has(skill.name))
  ]
}

/** What a step is worked with: the step's own skills, then everything its task already carries. */
export function stepSkills(
  stepId: TaskStepId,
  taskLevel: readonly EffectiveSkill[],
  owned: readonly TaskSkill[]
): readonly EffectiveSkill[] {
  const wider = new Set(taskLevel.map((skill) => skill.name))
  const own = owned
    .filter((skill) => skill.stepId === stepId && !wider.has(skill.skillName))
    .map((skill): EffectiveSkill => ({ name: skill.skillName, origin: 'step' }))
  return [...own, ...taskLevel]
}

const ORIGIN_LABEL: Readonly<Record<SkillOrigin, string>> = {
  step: 'this step',
  task: 'this task',
  tag: 'its tag'
}

/** One line per skill for a tooltip or a screen reader: `code-review, from this task`. */
export function describeSkills(skills: readonly EffectiveSkill[]): string {
  return skills.map((skill) => `${skill.name}, from ${ORIGIN_LABEL[skill.origin]}`).join('\n')
}
