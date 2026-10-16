<script setup lang="ts">
import SkillGlyph from '@/features/skills/components/SkillGlyph.vue'
import type { SkillOrigin } from '@/features/skills/model/skill-assignment'

/**
 * One assigned skill. A skill that comes from the tag is drawn dashed and cannot be removed
 * here, since taking it off means editing the tag; the border, not the colour, tells them apart.
 */
const props = withDefaults(
  defineProps<{
    name: string
    origin?: SkillOrigin
    /** The skill is no longer under `~/.claude`, so a session could not invoke it. */
    missing?: boolean
    removable?: boolean
  }>(),
  { origin: 'task', missing: false, removable: true }
)

defineEmits<{ remove: [] }>()

const originNote: Record<SkillOrigin, string> = {
  step: 'assigned to this step',
  task: 'assigned to this task',
  tag: "from the task's tag"
}
</script>

<template>
  <span
    class="inline-flex h-6 min-w-0 max-w-full items-center gap-1 rounded-key border pl-1.5 font-mono text-caption"
    :class="[
      props.removable && props.origin !== 'tag' ? 'pr-0.5' : 'pr-1.5',
      props.origin === 'tag'
        ? 'border-dashed border-border-strong text-text-muted'
        : 'border-anchor-line bg-anchor-soft text-anchor',
      props.missing && 'border-danger/50 text-danger'
    ]"
    :title="
      props.missing
        ? `${props.name} is not in ~/.claude, ${originNote[props.origin]}`
        : `${props.name}, ${originNote[props.origin]}`
    "
    data-testid="skill-chip"
    :data-origin="props.origin"
  >
    <SkillGlyph :size="11" />
    <span class="truncate">{{ props.name }}</span>
    <button
      v-if="props.removable && props.origin !== 'tag'"
      type="button"
      class="focus-ring grid size-5 shrink-0 place-items-center rounded-key text-current opacity-70 transition-opacity hover:opacity-100"
      :aria-label="`Remove skill ${props.name}`"
      data-testid="skill-chip-remove"
      @click="$emit('remove')"
    >
      <svg class="size-2.5" viewBox="0 0 10 10" fill="none" aria-hidden="true">
        <path d="M2 2l6 6M8 2l-6 6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
    </button>
  </span>
</template>
