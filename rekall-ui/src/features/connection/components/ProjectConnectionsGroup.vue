<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import ConnectionDialog from '@/features/connection/components/ConnectionDialog.vue'
import JiraImportDialog from '@/features/connection/components/JiraImportDialog.vue'
import ProjectConnectionRow from '@/features/connection/components/ProjectConnectionRow.vue'
import InspectorGroup from '@/features/project/components/InspectorGroup.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConnectionStream } from '@/features/connection/composables/useConnectionStream'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { Connection, ProjectConnection } from '@/features/connection/model/connection'
import { asConnectionId } from '@/model/branded'

/**
 * The connections this project's sessions reach: which of its company's it uses, and the GitLab
 * project path, namespace or Jira project key each is narrowed to. A missing credential is flagged here, since a
 * session on this project is told the same.
 */
const props = defineProps<{ project: Project }>()

const store = useConnectionsStore()
const { run } = useAsyncAction()
const chosen = ref<string | null>(null)
const creating = ref(false)
const importing = ref<ProjectConnection | null>(null)
const group = ref<InstanceType<typeof InspectorGroup> | null>(null)

const bindings = computed(() => store.projectConnectionsOf(props.project.id))
const unbound = computed(() => {
  const used = new Set(bindings.value.map((binding) => binding.connectionId))
  return store.connectionsOf(props.project.companyId).filter((connection) => !used.has(connection.id))
})
const options = computed(() =>
  unbound.value.map((connection) => ({
    value: connection.id as string,
    label: `${connection.title} · ${connection.label}`
  }))
)
const missing = computed(() => bindings.value.filter((binding) => !binding.credentialSetAt).length)
const summary = computed(() => {
  if (!bindings.value.length) return 'None used'
  const labels = bindings.value.map((binding) => binding.label).join(', ')
  return missing.value ? `${labels} · ${missing.value} without a credential` : labels
})

async function load(): Promise<void> {
  await run(() => Promise.all([store.loadProject(props.project.id), store.load(props.project.companyId)]))
}

watch(() => props.project.id, load, { immediate: true })

useConnectionStream(async (event) => {
  if (event.companyId === props.project.companyId) await load()
})

async function bind(connectionId: string, target: string | null = null): Promise<void> {
  await run(() => store.bind(props.project.id, props.project.companyId, asConnectionId(connectionId), target))
  chosen.value = null
}

async function setTarget(binding: ProjectConnection, target: string | null): Promise<void> {
  await run(() => store.bind(props.project.id, props.project.companyId, binding.connectionId, target))
}

async function unbind(binding: ProjectConnection): Promise<void> {
  await run(
    () => store.unbind(props.project.id, props.project.companyId, binding.connectionId),
    `${props.project.title} no longer uses ${binding.title}`
  )
}

async function bindCreated(connection: Connection): Promise<void> {
  await bind(connection.id)
}

function startCreating(): void {
  group.value?.expand()
  creating.value = true
}
</script>

<template>
  <InspectorGroup
    ref="group"
    group="connections"
    title="Connections"
    :count="bindings.length"
    :summary="summary"
  >
    <template #actions>
      <AppButton size="sm" variant="ghost" data-testid="new-project-connection" @click="startCreating"
        >+ New</AppButton
      >
    </template>

    <ul v-if="bindings.length" class="divide-y divide-border">
      <ProjectConnectionRow
        v-for="binding in bindings"
        :key="binding.connectionId"
        :binding="binding"
        @target="setTarget(binding, $event)"
        @unbind="unbind(binding)"
        @import-issues="importing = binding"
      />
    </ul>
    <p v-else class="text-caption text-text-subtle">
      No connection yet. A session on this project reaches only the clusters, GitLab instances and Jira sites added here.
    </p>

    <div v-if="options.length" class="mt-2 flex items-center gap-2">
      <div class="min-w-0 flex-1">
        <AppSelect
          v-model="chosen"
          :options="options"
          placeholder="Use a company connection"
          data-testid="connection-picker"
        />
      </div>
      <AppButton
        size="sm"
        variant="secondary"
        :disabled="!chosen"
        data-testid="bind-connection"
        @click="chosen && bind(chosen)"
        >Use</AppButton
      >
    </div>

    <RouterLink
      :to="`/companies/${project.companyId}/connections`"
      class="focus-ring mt-2 inline-block rounded-key text-caption text-text-subtle hover:text-accent"
      data-testid="company-connections-link"
    >
      Credentials, checks and the access log are on the company's Connections screen &#8594;
    </RouterLink>

    <Transition name="dialog">
      <ConnectionDialog
        v-if="creating"
        :company-id="project.companyId"
        :connection="null"
        @close="creating = false"
        @saved="bindCreated"
      />
    </Transition>
    <Transition name="dialog">
      <JiraImportDialog v-if="importing" :binding="importing" @close="importing = null" />
    </Transition>
  </InspectorGroup>
</template>
