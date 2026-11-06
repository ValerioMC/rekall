<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { TARGET_FIELD } from '@/features/connection/lib/connection-fields'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { ConnectionKind } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectId } from '@/model/branded'

/**
 * Binds one project of a company to the connection, narrowed to a target when one is typed.
 * A project may be chosen again with another target: it then uses the connection once more.
 */
const props = defineProps<{
  connectionId: ConnectionId
  connectionTitle: string
  kind: ConnectionKind
  projects: readonly { readonly id: ProjectId; readonly title: string }[]
}>()
const emit = defineEmits<{ close: []; bound: [] }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()
const projectId = ref<string | null>(null)
const target = ref('')
const picker = ref<HTMLElement | null>(null)

const targetField = computed(() => TARGET_FIELD[props.kind])
const options = computed(() => props.projects.map((project) => ({ value: project.id, label: project.title })))

async function bind(): Promise<void> {
  const project = props.projects.find((known) => known.id === projectId.value)
  if (!project || isRunning.value) return
  const bound = await run(
    () => store.bind(project.id, props.connectionId, { target: target.value.trim() || null }),
    `${project.title} now uses ${props.connectionTitle}`
  )
  if (bound) emit('bound')
}

onMounted(async () => {
  await nextTick()
  picker.value?.querySelector('select')?.focus()
})
</script>

<template>
  <form class="mt-2 flex flex-col gap-3 rounded-control border border-border-strong bg-surface px-3 py-3" data-testid="project-binder" @submit.prevent="bind" @keydown.esc.stop="emit('close')">
    <div ref="picker">
      <AppField v-slot="{ fieldId, describedBy }" label="Project" required>
        <AppSelect :id="fieldId" v-model="projectId" :described-by="describedBy" :options="options" placeholder="Choose a project" />
      </AppField>
    </div>
    <AppField
      v-slot="{ fieldId, describedBy }"
      :label="targetField.name"
      hint="Optional. Leave it empty to give the project the whole connection; type one to narrow it."
    >
      <AppInput :id="fieldId" v-model="target" :described-by="describedBy" mono :placeholder="targetField.placeholder" data-testid="binder-target" />
    </AppField>
    <div class="flex items-center justify-end gap-2">
      <AppButton size="sm" variant="ghost" class="max-sm:min-h-11" @click="emit('close')">Cancel</AppButton>
      <AppButton size="sm" variant="primary" type="submit" class="max-sm:min-h-11" :disabled="!projectId" :loading="isRunning" data-testid="binder-submit"
        >Use in project</AppButton
      >
    </div>
  </form>
</template>
