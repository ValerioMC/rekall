<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionDialog from '@/features/connection/components/ConnectionDialog.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import ConnectionTrunk from '@/features/connection/components/ConnectionTrunk.vue'
import JiraImportDialog from '@/features/connection/components/JiraImportDialog.vue'
import RepositoryDiscoveryResult from '@/features/connection/components/RepositoryDiscoveryResult.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { endpointOf } from '@/features/connection/lib/connection-endpoint'
import { connectionTrunks, trunkMatching, unusedConnections } from '@/features/connection/lib/connection-trunks'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import type { Project } from '@/features/catalog/model/catalog'
import type {
  Connection,
  ProjectConnection,
  ProjectConnectionInput,
  RepositoryDiscovery
} from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'

/**
 * The systems this project's sessions reach, as the company's connections with the bindings the
 * project hangs off each: a label a session names it by and the GitLab project path, namespace or
 * Jira project key it is narrowed to. The company's other connections wait below, one click from
 * use, and the repositories under the project's folder can be bound in one go.
 */
const props = defineProps<{ project: Project }>()

const store = useConnectionsStore()
const components = useProjectComponentsStore()
const { run } = useAsyncAction()
const { run: runDiscovery, isRunning: isDiscovering } = useAsyncAction()
const discovery = ref<RepositoryDiscovery | null>(null)
const query = ref('')
const creating = ref(false)
const importing = ref<ProjectConnection | null>(null)

const bindings = computed(() => store.projectConnectionsOf(props.project.id))
const companyConnections = computed(() => store.connectionsOf(props.project.companyId))
const trunks = computed(() =>
  connectionTrunks(bindings.value, companyConnections.value)
    .map((trunk) => trunkMatching(trunk, query.value))
    .filter((trunk) => trunk !== null)
)
const unused = computed(() => unusedConnections(bindings.value, companyConnections.value))

async function bind(connectionId: ConnectionId): Promise<void> {
  await run(() => store.bind(props.project.id, props.project.companyId, connectionId, { target: null }))
}

async function save(binding: ProjectConnection, input: ProjectConnectionInput): Promise<void> {
  await run(() => store.updateBinding(props.project.id, binding.id, input))
}

async function unbind(binding: ProjectConnection): Promise<void> {
  await run(
    () => store.unbind(props.project.id, props.project.companyId, binding.id),
    `${props.project.title} no longer uses ${binding.label}`
  )
}

/** The components list is read again too, since discovery adds a component per repository. */
async function discover(): Promise<void> {
  const found = await runDiscovery(async () => {
    const result = await store.discover(props.project.id, props.project.companyId)
    await components.load(props.project.id)
    return result
  })
  if (found) discovery.value = found
}

async function bindCreated(connection: Connection): Promise<void> {
  await bind(connection.id)
}

watch(
  () => props.project.id,
  () => {
    discovery.value = null
    query.value = ''
  }
)
</script>

<template>
  <section
    class="flex h-full min-h-0 flex-col overflow-hidden rounded-card border border-border bg-surface shadow-[var(--shadow-raised)]"
    aria-label="Connections"
    data-testid="project-connections"
  >
    <header class="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-border px-5 py-3">
      <div class="min-w-[240px] flex-1">
        <h2 class="text-title font-semibold tracking-[-0.01em] text-text">Connections</h2>
        <p class="mt-0.5 text-small text-text-subtle">
          What a session on this project can reach. A credential is stored once, on the company's connection, and every
          binding below it uses it.
        </p>
      </div>
      <label v-if="bindings.length" class="sr-only" for="connections-filter">Filter bindings</label>
      <input
        v-if="bindings.length"
        id="connections-filter"
        v-model="query"
        type="search"
        autocomplete="off"
        spellcheck="false"
        placeholder="Filter by label or target"
        class="field text-text h-8 w-[240px] max-w-full rounded-control px-2.5 text-small"
        data-testid="connections-filter"
      />
      <AppButton
        v-if="project.repoFolder"
        size="sm"
        variant="secondary"
        :disabled="isDiscovering"
        title="Bind every git repository under the project folder whose origin is on a GitLab with a token"
        data-testid="discover-repositories"
        @click="discover"
        >{{ isDiscovering ? 'Finding…' : 'Find repositories' }}</AppButton
      >
      <AppButton size="sm" variant="primary" data-testid="new-project-connection" @click="creating = true">New connection</AppButton>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto px-5 py-4">
      <RepositoryDiscoveryResult v-if="discovery" class="mb-5" :discovery="discovery" @dismiss="discovery = null" />

      <div v-if="trunks.length" class="flex flex-col gap-5">
        <ConnectionTrunk
          v-for="trunk in trunks"
          :key="trunk.connectionId"
          :trunk="trunk"
          @add-target="bind(trunk.connectionId)"
          @save="save"
          @unbind="unbind"
          @import-issues="importing = $event"
        />
      </div>
      <p v-else-if="bindings.length" class="text-small text-text-subtle" data-testid="connections-no-match">
        No binding matches “{{ query.trim() }}”.
      </p>
      <p v-else class="max-w-[60ch] text-small leading-relaxed text-text-subtle" data-testid="connections-empty">
        No connection yet. A session on this project reaches only the clusters, GitLab instances and Jira sites used
        here.
      </p>

      <div v-if="unused.length" class="mt-8" data-testid="unused-connections">
        <h3 class="section-label mb-1.5">Not used by this project</h3>
        <ul class="flex flex-col">
          <li
            v-for="connection in unused"
            :key="connection.id"
            class="flex h-[38px] items-center gap-3 border-t border-border first:border-t-0"
          >
            <ConnectionKindIcon :kind="connection.kind" :size="14" />
            <span class="truncate text-small text-text-muted">{{ connection.title }}</span>
            <span class="hidden truncate font-mono text-caption text-text-subtle sm:inline">{{ endpointOf(connection) }}</span>
            <span v-if="!connection.credentialSetAt" class="text-caption text-warn">no credential</span>
            <AppButton
              class="ml-auto"
              size="sm"
              variant="secondary"
              :data-testid="`use-connection-${connection.id}`"
              @click="bind(connection.id)"
              >Use</AppButton
            >
          </li>
        </ul>
      </div>

      <RouterLink
        :to="`/companies/${project.companyId}/connections`"
        class="focus-ring mt-8 inline-block rounded-key text-caption text-text-subtle hover:text-accent"
        data-testid="company-connections-link"
      >
        Credentials, checks and the access log are on the company's Connections screen
      </RouterLink>
    </div>

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
  </section>
</template>
