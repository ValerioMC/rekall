<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import ConnectionOption from '@/features/connection/components/ConnectionOption.vue'
import ConnectionSheet from '@/features/connection/components/ConnectionSheet.vue'
import ConnectionTrunk from '@/features/connection/components/ConnectionTrunk.vue'
import JiraImportDialog from '@/features/connection/components/JiraImportDialog.vue'
import RepositoryDiscoveryResult from '@/features/connection/components/RepositoryDiscoveryResult.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { connectionTrunks, trunkMatching, trunksByKind, unusedConnections } from '@/features/connection/lib/connection-trunks'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { Project } from '@/features/catalog/model/catalog'
import { CONNECTION_KIND_LABEL, CONNECTION_KIND_NOUN } from '@/features/connection/model/connection'
import type {
  Connection,
  ProjectConnection,
  ProjectConnectionInput,
  RepositoryDiscovery
} from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'

/**
 * The systems this project's sessions reach, as the connections shared with its company and the
 * bindings the project hangs off each: a label a session names it by and the GitLab project path,
 * GitHub repository, namespace or Jira project key it is narrowed to. This screen only binds: a
 * credential, a check and the companies a connection is shared with are changed in the
 * Connections screen, one link away. A connection can be created here, shared with the project's
 * company and bound in one go.
 */
const props = defineProps<{ project: Project }>()

const FILTER_FROM = 7

const store = useConnectionsStore()
const consoleStore = useConsoleStore()
const toast = useToastStore()
const components = useProjectComponentsStore()
const { run } = useAsyncAction()
const { run: runDiscovery, isRunning: isDiscovering } = useAsyncAction()
const discovery = ref<RepositoryDiscovery | null>(null)
const query = ref('')
const creating = ref(false)
const importing = ref<ProjectConnection | null>(null)
const returnTo = computed(() => `/projects/${props.project.id}`)
const companies = computed(() => consoleStore.companies.map((company) => ({ id: company.id, name: company.name })))
const hubQuery = computed(() => ({ company: props.project.companyId, returnTo: returnTo.value }))

const bindings = computed(() => store.projectConnectionsOf(props.project.id))
const companyConnections = computed(() => store.connectionsOf(props.project.companyId))
const kindGroups = computed(() =>
  trunksByKind(
    connectionTrunks(bindings.value, companyConnections.value)
      .map((trunk) => trunkMatching(trunk, query.value))
      .filter((trunk) => trunk !== null)
  )
)
const unused = computed(() => unusedConnections(bindings.value, companyConnections.value))
const unsharedElsewhere = computed(() => store.connections.length - companyConnections.value.length)

async function bind(connectionId: ConnectionId): Promise<void> {
  await run(() => store.bind(props.project.id, connectionId, { target: null }))
}

async function save(binding: ProjectConnection, input: ProjectConnectionInput): Promise<void> {
  await run(() => store.updateBinding(props.project.id, binding.id, input))
}

async function unbind(binding: ProjectConnection): Promise<void> {
  await run(
    () => store.unbind(props.project.id, binding.id),
    `${props.project.title} no longer uses ${binding.label}`
  )
}

/** The components list is read again too, since discovery adds a component per repository. */
async function discover(): Promise<void> {
  const found = await runDiscovery(async () => {
    const result = await store.discover(props.project.id)
    await components.load(props.project.id)
    return result
  })
  if (found) discovery.value = found
}

async function bindCreated(connection: Connection): Promise<void> {
  try {
    await store.bind(props.project.id, connection.id, { target: null })
    toast.notify(`${props.project.title} now uses ${connection.title}`)
  } catch (failure) {
    const reason = failure instanceof Error ? failure.message : 'the server refused it'
    toast.notifyError(new Error(`${connection.title} was created but not bound to ${props.project.title}: ${reason}`))
  }
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
          Connections this project can reach. Credentials are managed on the connection.
        </p>
      </div>
      <label v-if="bindings.length > FILTER_FROM" class="sr-only" for="connections-filter">Filter bindings</label>
      <input
        v-if="bindings.length > FILTER_FROM"
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
        title="Bind every git repository under the project folder whose origin is on a GitLab or GitHub with a token"
        data-testid="discover-repositories"
        @click="discover"
        >{{ isDiscovering ? 'Finding…' : 'Find repositories' }}</AppButton
      >
      <RouterLink
        :to="{ name: 'connections', query: hubQuery }"
        class="focus-ring key-slate inline-flex h-7 items-center rounded-key px-3 text-caption font-medium max-sm:min-h-11"
        data-testid="manage-connections-link"
        >Manage connections</RouterLink
      >
      <AppButton size="sm" variant="primary" data-testid="new-project-connection" @click="creating = true">+ New connection</AppButton>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto px-5 py-4">
      <RepositoryDiscoveryResult v-if="discovery" class="mb-5" :discovery="discovery" @dismiss="discovery = null" />

      <div v-if="kindGroups.length" class="flex flex-col gap-8">
        <section
          v-for="group in kindGroups"
          :key="group.kind"
          :aria-label="CONNECTION_KIND_LABEL[group.kind]"
          :data-testid="`connection-kind-${group.kind}`"
        >
          <h3 class="section-label mb-3 flex items-center gap-2 border-b border-border pb-1.5">
            <ConnectionKindIcon :kind="group.kind" :size="12" />
            {{ CONNECTION_KIND_LABEL[group.kind] }}
            <span class="font-mono tabular-nums text-text-subtle">
              {{ group.trunks.length }} {{ CONNECTION_KIND_NOUN[group.kind] }}{{ group.trunks.length === 1 ? '' : 's' }}
            </span>
          </h3>
          <div class="flex flex-col gap-5">
            <ConnectionTrunk
              v-for="trunk in group.trunks"
              :key="trunk.connectionId"
              :trunk="trunk"
              :return-to="returnTo"
              @add-target="bind(trunk.connectionId)"
              @save="save"
              @unbind="unbind"
              @import-issues="importing = $event"
            />
          </div>
        </section>
      </div>
      <p v-else-if="bindings.length" class="text-small text-text-subtle" data-testid="connections-no-match">
        No binding matches “{{ query.trim() }}”.
      </p>
      <div v-else-if="!companyConnections.length" class="max-w-[60ch]" data-testid="no-shared-connections">
        <p class="text-small font-medium text-text">No connection is shared with {{ project.companyName }} yet</p>
        <p class="mt-1 text-small leading-relaxed text-text-subtle">
          A session on this project reaches only the connections its company shares. Create one here, or share one that
          already exists.
        </p>
        <div class="mt-3 flex flex-wrap items-center gap-3">
          <AppButton size="sm" variant="secondary" class="max-sm:min-h-11" data-testid="empty-new-connection" @click="creating = true"
            >+ New connection</AppButton
          >
          <RouterLink
            :to="{ name: 'connections', query: hubQuery }"
            class="focus-ring inline-flex min-h-11 items-center rounded-key text-small text-accent hover:underline sm:min-h-0"
            data-testid="shared-connections-link"
            >Share one from Connections</RouterLink
          >
        </div>
      </div>
      <p v-else class="max-w-[60ch] text-small leading-relaxed text-text-subtle" data-testid="connections-empty">
        No connection used yet. A session on this project reaches only the clusters, GitLab instances, GitHub servers, Jira sites and Grafana instances
        bound here.
      </p>

      <div v-if="unused.length" class="mt-8" data-testid="unused-connections">
        <h3 class="section-label mb-1.5">Not used by this project</h3>
        <ul class="flex flex-col">
          <li v-for="connection in unused" :key="connection.id" class="border-t border-border first:border-t-0">
            <ConnectionOption :connection="connection" :return-to="returnTo" @use="bind(connection.id)" />
          </li>
        </ul>
      </div>

      <p v-if="unsharedElsewhere > 0" class="mt-6 text-small text-text-subtle" data-testid="unshared-elsewhere">
        {{ unsharedElsewhere }} more {{ unsharedElsewhere === 1 ? 'connection exists' : 'connections exist' }} but
        {{ unsharedElsewhere === 1 ? 'is' : 'are' }} not shared with {{ project.companyName }}.
        <RouterLink :to="{ name: 'connections', query: { returnTo } }" class="focus-ring rounded-key text-accent hover:underline"
          >See them in Connections</RouterLink
        >
      </p>
    </div>

    <Transition name="slide-over">
      <ConnectionSheet
        v-if="creating"
        :connection="null"
        :initial-kind="null"
        :initial-company-ids="[project.companyId]"
        :bind-to-project="project.title"
        :companies="companies"
        @close="creating = false"
        @saved="bindCreated"
      />
    </Transition>
    <Transition name="dialog">
      <JiraImportDialog v-if="importing" :binding="importing" @close="importing = null" />
    </Transition>
  </section>
</template>
