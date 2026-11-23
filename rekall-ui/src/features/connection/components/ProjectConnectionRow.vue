<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import GitlabTargetEditor from '@/features/connection/components/GitlabTargetEditor.vue'
import ConnectionUnbindButton from '@/features/connection/components/ConnectionUnbindButton.vue'
import TargetSuggestionList from '@/features/connection/components/TargetSuggestionList.vue'
import { useTargetSuggestions } from '@/features/connection/composables/useTargetSuggestions'
import { TARGET_FIELD } from '@/features/connection/lib/connection-fields'
import { expectedTargetPrefix } from '@/features/connection/lib/target-scope'
import type { ProjectConnection, ProjectConnectionInput } from '@/features/connection/model/connection'

/**
 * One binding on its connection's line: the label a session names it by and its target, both
 * edited in place, and a way to stop using it. A GitLab target is changed in the path field a new
 * binding gets, so it can only be one GitLab confirms. A Jira or Confluence one with a credential offers the
 * projects or spaces of its site while the target is typed; a Jira one with a project key also
 * opens the import of its issues and a Confluence one with a space key the filling of the project.
 */
const props = defineProps<{ binding: ProjectConnection; rootPath: string | null }>()
const emit = defineEmits<{ save: [input: ProjectConnectionInput]; unbind: []; importIssues: []; fillProject: [] }>()

const changingGitlabTarget = ref(false)
const isGitlab = computed(() => props.binding.kind === 'GITLAB')
const draft = ref(props.binding.target ?? '')
const labelDraft = ref(props.binding.label)
const targetField = computed(() => TARGET_FIELD[props.binding.kind])
const expectedPrefix = computed(() => expectedTargetPrefix(props.rootPath, draft.value.trim() || null))
const canImport = computed(
  () => props.binding.kind === 'JIRA' && !!props.binding.credentialSetAt && !!props.binding.target
)

const takesSuggestions = computed(
  () => (props.binding.kind === 'JIRA' || props.binding.kind === 'CONFLUENCE') && !!props.binding.credentialSetAt
)
const canFillProject = computed(
  () => props.binding.kind === 'CONFLUENCE' && !!props.binding.credentialSetAt && !!props.binding.target
)
const suggestionListId = computed(() => `target-suggestions-${props.binding.id}`)
const { suggestions, unavailable, ask } = useTargetSuggestions(() => props.binding.connectionId, () => takesSuggestions.value)

watch(draft, (typed) => ask(typed))

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

function saveGitlabTarget(target: string | null): void {
  changingGitlabTarget.value = false
  emit('save', { label: props.binding.label, target })
}

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
  <div>
    <div class="group flex min-h-(--spacing-row) items-center gap-2 rounded-control pr-1 hover:bg-surface-raised" data-testid="project-connection">
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
      <template v-if="isGitlab">
        <span class="text-text min-w-0 flex-1 truncate font-mono text-caption" data-testid="connection-target">{{ binding.target ?? 'Whole connection' }}</span>
        <AppButton size="sm" variant="ghost" data-testid="change-gitlab-target" @click="changingGitlabTarget = true">Change</AppButton>
      </template>
      <input
        v-else
        v-model="draft"
        spellcheck="false"
        autocomplete="off"
        :aria-label="`${targetField.name} for ${binding.title}`"
        data-testid="connection-target"
        class="quiet-input text-text min-w-0 flex-1 font-mono text-caption"
        :placeholder="targetField.placeholder"
        :list="takesSuggestions ? suggestionListId : undefined"
        @focus="ask(draft)"
        @blur="commit"
        @keydown.enter.prevent="commit"
      />
      <TargetSuggestionList v-if="takesSuggestions" :id="suggestionListId" :suggestions="suggestions" />
      <span v-if="unavailable" class="shrink-0 text-caption text-text-subtle" role="status" data-testid="suggestions-unavailable"
        >Suggestions unavailable: {{ unavailable }}</span
      >
      <span v-if="expectedPrefix" class="shrink-0 text-caption text-warn" role="status" data-testid="target-warning"
        >Expected to start with {{ expectedPrefix }}</span
      >
      <AppButton v-if="canImport" size="sm" variant="ghost" data-testid="import-jira-issues" @click="emit('importIssues')"
        >Import issues as tasks</AppButton
      >
      <AppButton v-if="canFillProject" size="sm" variant="ghost" data-testid="fill-from-confluence" @click="emit('fillProject')"
        >Fill project from Confluence</AppButton
      >
      <ConnectionUnbindButton :subject="binding.label" testid="unbind-connection" @confirm="emit('unbind')" />
    </div>
    <GitlabTargetEditor
      v-if="changingGitlabTarget"
      :binding="binding"
      :root-path="rootPath"
      @save="saveGitlabTarget"
      @cancel="changingGitlabTarget = false"
    />
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
