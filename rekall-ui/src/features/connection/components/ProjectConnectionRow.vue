<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { TARGET_FIELD } from '@/features/connection/lib/connection-fields'
import type { ProjectConnection, ProjectConnectionInput } from '@/features/connection/model/connection'

/**
 * One connection the project uses: its label and target edited in place, and a way to stop using
 * it. A Jira one with a credential and a project key also opens the import of its issues.
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
  <li class="flex flex-col gap-1.5 py-2" data-testid="project-connection">
    <div class="flex items-center gap-2">
      <ConnectionKindIcon :kind="binding.kind" :size="14" />
      <span class="min-w-0 flex-1 truncate text-small text-text">{{ binding.title }}</span>
      <AppBadge v-if="!binding.credentialSetAt" tone="warn" dot data-testid="project-connection-missing"
        >No credential</AppBadge
      >
      <button
        class="focus-ring grid size-6 place-items-center rounded-key text-text-subtle hover:bg-surface-hover hover:text-danger"
        :aria-label="`Stop using ${binding.title}`"
        data-testid="unbind-connection"
        @click="emit('unbind')"
      >
        <svg class="size-3" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="m3.5 3.5 7 7m0-7-7 7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
      </button>
    </div>
    <div class="flex items-center gap-2">
      <input
        v-model="labelDraft"
        spellcheck="false"
        autocomplete="off"
        :aria-label="`Label a session names ${binding.title} by`"
        data-testid="connection-label"
        class="field h-7 w-32 shrink-0 rounded-control px-2 font-mono text-caption text-anchor/80"
        @blur="commitLabel"
        @keydown.enter.prevent="commitLabel"
      />
      <input
        v-model="draft"
        spellcheck="false"
        autocomplete="off"
        :aria-label="`${targetField.name} for ${binding.title}`"
        data-testid="connection-target"
        class="field text-text h-7 min-w-0 flex-1 rounded-control px-2 font-mono text-caption"
        :placeholder="targetField.placeholder"
        @blur="commit"
        @keydown.enter.prevent="commit"
      />
    </div>
    <div v-if="canImport" class="flex justify-end">
      <AppButton size="sm" variant="ghost" data-testid="import-jira-issues" @click="emit('importIssues')"
        >Import issues as tasks</AppButton
      >
    </div>
  </li>
</template>
