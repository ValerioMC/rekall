<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { TARGET_FIELD } from '@/features/connection/lib/connection-fields'
import type { ProjectConnection, ProjectConnectionInput } from '@/features/connection/model/connection'

/**
 * One binding on its connection's line: the label a session names it by and its target, both
 * edited in place, and a way to stop using it. A Jira one with a credential and a project key
 * also opens the import of its issues.
 */
const props = defineProps<{ binding: ProjectConnection }>()
const emit = defineEmits<{ save: [input: ProjectConnectionInput]; unbind: []; importIssues: [] }>()

const draft = ref(props.binding.target ?? '')
const labelDraft = ref(props.binding.label)
const targetField = computed(() => TARGET_FIELD[props.binding.kind])
const canImport = computed(
  () => props.binding.kind === 'JIRA' && !!props.binding.credentialSetAt && !!props.binding.target
)

watch(
  () => props.binding.target,
  (target) => {
    draft.value = target ?? ''
  }
)

watch(
  () => props.binding.label,
  (label) => {
    labelDraft.value = label
  }
)

function commit(): void {
  const target = draft.value.trim() || null
  if (target !== props.binding.target) emit('save', { label: props.binding.label, target })
}

/** A blank label goes back to the one the binding has; the server makes a typed one a slug. */
function commitLabel(): void {
  const label = labelDraft.value.trim()
  if (!label) {
    labelDraft.value = props.binding.label
    return
  }
  if (label !== props.binding.label) emit('save', { label, target: props.binding.target })
}
</script>

<template>
  <div class="group flex h-[34px] items-center gap-2 rounded-control pr-1 hover:bg-surface-raised" data-testid="project-connection">
    <input
      v-model="labelDraft"
      spellcheck="false"
      autocomplete="off"
      :aria-label="`Label a session names ${binding.title} by`"
      data-testid="connection-label"
      class="quiet-input w-44 shrink-0 font-mono text-caption text-anchor"
      @blur="commitLabel"
      @keydown.enter.prevent="commitLabel"
    />
    <input
      v-model="draft"
      spellcheck="false"
      autocomplete="off"
      :aria-label="`${targetField.name} for ${binding.title}`"
      data-testid="connection-target"
      class="quiet-input text-text min-w-0 flex-1 font-mono text-caption"
      :placeholder="targetField.placeholder"
      @blur="commit"
      @keydown.enter.prevent="commit"
    />
    <AppButton v-if="canImport" size="sm" variant="ghost" data-testid="import-jira-issues" @click="emit('importIssues')"
      >Import issues as tasks</AppButton
    >
    <button
      class="focus-ring grid size-7 shrink-0 cursor-pointer place-items-center rounded-key text-text-subtle opacity-60 transition-opacity hover:bg-surface-hover hover:text-danger hover:opacity-100 focus-visible:opacity-100 group-hover:opacity-100"
      :aria-label="`Stop using ${binding.label}`"
      :title="`Stop using ${binding.label}`"
      data-testid="unbind-connection"
      @click="emit('unbind')"
    >
      <svg class="size-3" viewBox="0 0 14 14" fill="none" aria-hidden="true">
        <path d="m3.5 3.5 7 7m0-7-7 7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
/* Reads as text until touched: no frame at rest, the field's own recess on hover and focus. */
.quiet-input {
  height: 28px;
  border: 1px solid transparent;
  border-radius: var(--radius-control);
  background: transparent;
  padding: 0 8px;
  transition:
    border-color var(--motion-quick) ease,
    background-color var(--motion-quick) ease,
    box-shadow var(--motion-quick) ease;
}

.quiet-input::placeholder {
  color: var(--color-text-subtle);
}

.quiet-input:hover {
  border-color: var(--color-border);
}

.quiet-input:focus {
  outline: none;
  background: var(--color-canvas);
  border-color: color-mix(in srgb, var(--color-accent) 62%, transparent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-accent) 14%, transparent);
}
</style>
