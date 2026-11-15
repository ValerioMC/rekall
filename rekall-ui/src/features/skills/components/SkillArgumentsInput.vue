<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import { SKILL_ARGUMENTS_MAX } from '@/features/skills/model/skill-assignment'

/**
 * The free text one skill is invoked with. The text is kept here while it is typed and handed
 * up trimmed when the field is left or the panel closes, so a list is saved once per edit and
 * not once per keystroke. A blank text means no arguments.
 */
const props = defineProps<{ name: string; modelValue: string | null }>()

const emit = defineEmits<{ 'update:modelValue': [value: string | null] }>()

const draft = ref(props.modelValue ?? '')

watch(
  () => props.modelValue,
  (value) => (draft.value = value ?? '')
)

function commit(): void {
  const trimmed = draft.value.trim()
  const value = trimmed === '' ? null : trimmed
  draft.value = value ?? ''
  if (value !== props.modelValue) emit('update:modelValue', value)
}

onBeforeUnmount(commit)
</script>

<template>
  <input
    v-model="draft"
    type="text"
    autocomplete="off"
    spellcheck="false"
    :maxlength="SKILL_ARGUMENTS_MAX"
    placeholder="Arguments, optional: redesign website ui"
    :aria-label="`Arguments for ${name}`"
    class="focus-ring h-7 w-full min-w-0 rounded-control border border-border bg-canvas px-2.5 font-mono text-caption text-text placeholder:font-sans placeholder:text-text-subtle"
    data-testid="skill-arguments"
    :data-arguments-for="name"
    @change="commit"
  />
</template>
