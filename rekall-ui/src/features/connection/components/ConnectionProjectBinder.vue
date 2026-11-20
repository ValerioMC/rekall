<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import GitlabPathField from '@/features/connection/components/GitlabPathField.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import TargetSuggestionList from '@/features/connection/components/TargetSuggestionList.vue'
import { useTargetSuggestions } from '@/features/connection/composables/useTargetSuggestions'
import { TARGET_FIELD } from '@/features/connection/lib/connection-fields'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { ConnectionKind } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectId } from '@/model/branded'

/**
 * Binds one project of a company to the connection, narrowed to a target when one is typed.
 * A project may be chosen again with another target: it then uses the connection once more. On
 * GitLab the target is typed or picked from what the token can read and has to exist before the
 * project can be bound. With a locked project the choice is not offered: the binder serves that
 * project's own page.
 */
const props = defineProps<{
  connectionId: ConnectionId
  connectionTitle: string
  kind: ConnectionKind
  credentialSet: boolean
  baseUrl: string | null
  rootPath: string | null
  lockedProjectId?: ProjectId
  projects: readonly { readonly id: ProjectId; readonly title: string }[]
}>()
const emit = defineEmits<{ close: []; bound: [] }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()
const projectId = ref<string | null>(props.lockedProjectId ?? null)
const target = ref('')
const form = ref<HTMLElement | null>(null)
const gitlabBinding = ref<{ readonly ready: boolean; readonly path: string | null }>({ ready: true, path: null })

const targetField = computed(() => TARGET_FIELD[props.kind])
const isGitlab = computed(() => props.kind === 'GITLAB')
const canBind = computed(() => !!projectId.value && (!isGitlab.value || gitlabBinding.value.ready))
const takesSuggestions = computed(() => (props.kind === 'JIRA' || props.kind === 'CONFLUENCE') && props.credentialSet)
const suggestionListId = computed(() => `target-suggestions-${props.connectionId}`)
const { suggestions, unavailable, ask } = useTargetSuggestions(() => props.connectionId, () => takesSuggestions.value)
const options = computed(() => props.projects.map((project) => ({ value: project.id, label: project.title })))

watch(target, (typed) => ask(typed))

function chosenTarget(): string | null {
  if (isGitlab.value) return gitlabBinding.value.path
  return target.value.trim() || null
}

async function bind(): Promise<void> {
  const project = props.projects.find((known) => known.id === projectId.value)
  if (!project || isRunning.value) return
  const bound = await run(
    () => store.bind(project.id, props.connectionId, { target: chosenTarget() }),
    `${project.title} now uses ${props.connectionTitle}`
  )
  if (bound) emit('bound')
}

onMounted(async () => {
  await nextTick()
  form.value?.querySelector<HTMLElement>('select, input')?.focus()
})
</script>

<template>
  <form ref="form" class="mt-2 flex flex-col gap-3 rounded-control border border-border-strong bg-surface px-3 py-3" data-testid="project-binder" @submit.prevent="bind" @keydown.esc.stop="emit('close')">
    <div v-if="!lockedProjectId">
      <AppField v-slot="{ fieldId, describedBy }" label="Project" required>
        <AppSelect :id="fieldId" v-model="projectId" :described-by="describedBy" :options="options" placeholder="Choose a project" />
      </AppField>
    </div>
    <GitlabPathField
      v-if="isGitlab"
      :connection-id="connectionId"
      :connection-title="connectionTitle"
      :base-url="baseUrl"
      :root-path="rootPath"
      @resolve="gitlabBinding = $event"
    />
    <AppField
      v-else
      v-slot="{ fieldId, describedBy }"
      :label="targetField.name"
      :hint="unavailable ? `Suggestions unavailable: ${unavailable}` : 'Optional. Leave it empty to give the project the whole connection; type one to narrow it.'"
    >
      <AppInput
        :id="fieldId"
        v-model="target"
        :described-by="describedBy"
        mono
        :placeholder="targetField.placeholder"
        :list="takesSuggestions ? suggestionListId : undefined"
        data-testid="binder-target"
        @focus="ask(target)"
      />
      <TargetSuggestionList v-if="takesSuggestions" :id="suggestionListId" :suggestions="suggestions" />
    </AppField>
    <div class="flex items-center justify-end gap-2">
      <AppButton size="sm" variant="ghost" class="max-sm:min-h-11" @click="emit('close')">Cancel</AppButton>
      <AppButton size="sm" variant="primary" type="submit" class="max-sm:min-h-11" :disabled="!canBind" :loading="isRunning" data-testid="binder-submit"
        >Use in project</AppButton
      >
    </div>
  </form>
</template>
