<script setup lang="ts">
import { computed } from 'vue'
import SkillGlyph from '@/features/skills/components/SkillGlyph.vue'
import { describeSkills, type EffectiveSkill } from '@/features/skills/model/skill-assignment'

/**
 * The small mark that says a task or step has skills: the spark and how many. The names and
 * where each comes from are its tooltip and its accessible name, so the count is not the only
 * way to read it. Nothing renders for none.
 */
const props = defineProps<{ skills: readonly EffectiveSkill[] }>()

const description = computed(() => describeSkills(props.skills))
const label = computed(() => {
  const count = props.skills.length
  return `${count} ${count === 1 ? 'skill' : 'skills'}: ${props.skills.map((skill) => skill.name).join(', ')}`
})
</script>

<template>
  <span
    v-if="skills.length"
    class="inline-flex h-[18px] shrink-0 items-center gap-1 rounded-full border border-anchor-line bg-anchor-soft px-1.5 text-micro font-semibold tabular-nums text-anchor"
    role="img"
    :aria-label="label"
    :title="description"
    data-testid="skill-mark"
  >
    <SkillGlyph :size="10" />
    {{ skills.length }}
  </span>
</template>
