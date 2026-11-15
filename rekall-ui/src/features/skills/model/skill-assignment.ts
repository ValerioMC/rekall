import type { ResourceSummary } from '@/features/claude-resources/model/claude-resources'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'

/** A skill and the free text it is invoked with. A skill is told apart by both together. */
export interface SkillAssignment {
  readonly name: string
  readonly arguments: string | null
}

/** A skill on a task (`stepId` null) or on one of its steps. */
export interface TaskSkill {
  readonly taskId: TaskId
  readonly stepId: TaskStepId | null
  readonly skillName: string
  readonly arguments: string | null
}

/** A skill every task carrying the tag uses by default. */
export interface TagSkill {
  readonly tagId: TagId
  readonly skillName: string
  readonly arguments: string | null
}

/** Where a skill on a task or step comes from, which decides whether it can be taken off there. */
export type SkillOrigin = 'step' | 'task' | 'tag'

export interface EffectiveSkill extends SkillAssignment {
  readonly origin: SkillOrigin
}

export const SKILLS_PER_OWNER_MAX = 20

/** What the server accepts as a skill's arguments, counted in UTF-16 units like the input's `maxlength`. */
export const SKILL_ARGUMENTS_MAX = 300

/** What a session types after the slash: a plugin's skill is `plugin:skill`, any other its own name. */
export function invocationName(skill: ResourceSummary): string {
  return skill.source?.kind === 'plugin' ? `${skill.source.name}:${skill.name}` : skill.name
}

/** What identifies an invocation: the same skill with other arguments is another one. */
export function invocationKey(skill: SkillAssignment): string {
  return JSON.stringify([skill.name, skill.arguments])
}

/** The skills a tag brings to every task under it. */
export function skillsOfTag(tagId: TagId | null, tagged: readonly TagSkill[]): readonly EffectiveSkill[] {
  if (tagId === null) return []
  return tagged
    .filter((skill) => skill.tagId === tagId)
    .map((skill): EffectiveSkill => ({ name: skill.skillName, arguments: skill.arguments, origin: 'tag' }))
}

/** The task's own skills, then its tag's; an invocation the task holds itself is not listed twice. */
export function taskSkills(
  taskId: TaskId,
  tagId: TagId | null,
  owned: readonly TaskSkill[],
  tagged: readonly TagSkill[]
): readonly EffectiveSkill[] {
  const own = owned.filter((skill) => skill.taskId === taskId && skill.stepId === null)
  const held = new Set(
    own.map((skill) => invocationKey({ name: skill.skillName, arguments: skill.arguments }))
  )
  return [
    ...own.map((skill): EffectiveSkill => ({
      name: skill.skillName,
      arguments: skill.arguments,
      origin: 'task'
    })),
    ...skillsOfTag(tagId, tagged).filter((skill) => !held.has(invocationKey(skill)))
  ]
}

/** What a step is worked with: the step's own skills, then everything its task already carries. */
export function stepSkills(
  stepId: TaskStepId,
  taskLevel: readonly EffectiveSkill[],
  owned: readonly TaskSkill[]
): readonly EffectiveSkill[] {
  const wider = new Set(taskLevel.map(invocationKey))
  const own = owned
    .filter((skill) => skill.stepId === stepId)
    .map((skill): EffectiveSkill => ({ name: skill.skillName, arguments: skill.arguments, origin: 'step' }))
    .filter((skill) => !wider.has(invocationKey(skill)))
  return [...own, ...taskLevel]
}

const ORIGIN_LABEL: Readonly<Record<SkillOrigin, string>> = {
  step: 'this step',
  task: 'this task',
  tag: 'its tag'
}

/** One line per skill for a tooltip or a screen reader: `impeccable (redesign), from this task`. */
export function describeSkills(skills: readonly EffectiveSkill[]): string {
  return skills.map((skill) => `${nameWithArguments(skill)}, from ${ORIGIN_LABEL[skill.origin]}`).join('\n')
}

function nameWithArguments(skill: SkillAssignment): string {
  return skill.arguments === null ? skill.name : `${skill.name} (${skill.arguments})`
}
