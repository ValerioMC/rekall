<script setup lang="ts">
import { computed } from 'vue'
import SkillGlyph from '@/features/skills/components/SkillGlyph.vue'
import type { SkillOrigin } from '@/features/skills/model/skill-assignment'

/**
 * One assigned skill. A skill that comes from the tag is drawn dashed and cannot be removed
 * here, since taking it off means editing the tag; the border, not the colour, tells them apart.
 */
const props = withDefaults(
  defineProps<{
    name: string
    /** The free text the skill is invoked with, drawn after the name. */
    arguments?: string | null
    origin?: SkillOrigin
    /** The name and arguments are a button that asks to edit them. */
    editable?: boolean
    /** The skill is no longer under `~/.claude`, so a session could not invoke it. */
    missing?: boolean
    removable?: boolean
  }>(),
  { arguments: null, origin: 'task', missing: false, removable: true, editable: false }
)

defineEmits<{ remove: []; edit: [] }>()

const originNote: Record<SkillOrigin, string> = {
  step: 'assigned to this step',
  task: 'assigned to this task',
  tag: "from the task's tag"
}

const description = computed(() => {
  const named = props.arguments ? `${props.name} ${props.arguments}` : props.name
  return props.missing
    ? `${named} is not in ~/.claude, ${originNote[props.origin]}`
    : `${named}, ${originNote[props.origin]}`
})
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
    :title="description"
    data-testid="skill-chip"
    :data-origin="props.origin"
  >
    <component
      :is="props.editable ? 'button' : 'span'"
      :type="props.editable ? 'button' : undefined"
      class="flex min-w-0 items-center gap-1"
      :class="props.editable && 'focus-ring -ml-1 rounded-key pl-1 transition-opacity hover:opacity-80'"
      :aria-label="props.editable ? `Edit arguments of skill ${props.name}` : undefined"
      :data-testid="props.editable ? 'skill-chip-edit' : undefined"
      @click="props.editable && $emit('edit')"
    >
      <SkillGlyph :size="11" />
      <span class="max-w-[26ch] shrink-0 truncate">{{ props.name }}</span>
      <template v-if="props.arguments">
        <span class="opacity-50" aria-hidden="true">·</span>
        <span class="max-w-[22ch] min-w-0 truncate font-sans opacity-80" data-testid="skill-chip-arguments">{{
          props.arguments
        }}</span>
      </template>
    </component>
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
