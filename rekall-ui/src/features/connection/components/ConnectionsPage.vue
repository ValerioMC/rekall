<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import ConnectionCompaniesDialog from '@/features/connection/components/ConnectionCompaniesDialog.vue'
import ConnectionDeleteDialog from '@/features/connection/components/ConnectionDeleteDialog.vue'
import ConnectionDetail from '@/features/connection/components/ConnectionDetail.vue'
import ConnectionEmptyState from '@/features/connection/components/ConnectionEmptyState.vue'
import ConnectionFilterBar from '@/features/connection/components/ConnectionFilterBar.vue'
import ConnectionList from '@/features/connection/components/ConnectionList.vue'
import ConnectionSheet from '@/features/connection/components/ConnectionSheet.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConnectionStream } from '@/features/connection/composables/useConnectionStream'
import { useConnectionsRoute } from '@/features/connection/composables/useConnectionsRoute'
import { useFocusReturn } from '@/features/connection/composables/useFocusReturn'
import { changeAnnouncement } from '@/features/connection/lib/connection-changes'
import {
  CONNECTION_STATUS_LABEL,
  activeFilterCount,
  filterConnections,
  sectionsByKind
} from '@/features/connection/lib/connection-filter'
import { returnLabelOf } from '@/features/connection/lib/connection-route'
import { CONNECTION_KIND_LABEL } from '@/features/connection/model/connection'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Connection, ConnectionKind } from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'

/**
 * The hub where every connection is configured once: the accounts and servers projects reach,
 * each shared with the companies that may bind it. The list and the selected connection sit side
 * by side, or one after the other on a narrow screen; the URL holds the selection and filters.
 */
const store = useConnectionsStore()
const consoleStore = useConsoleStore()
const { run } = useAsyncAction()
const { selectedId, filter, returnTo, select, setFilter, clearFilters } = useConnectionsRoute()
const sheetFocus = useFocusReturn()
const dialogFocus = useFocusReturn()

const loadFailed = ref(false)
const announcement = ref('')
const sheet = ref<{ connection: Connection | null; kind: ConnectionKind | null } | null>(null)
const editingCompanies = ref(false)
const deleting = ref<Connection | null>(null)
const detailPane = ref<InstanceType<typeof ConnectionDetail> | null>(null)

const visible = computed(() => filterConnections(store.connections, filter.value))
const sections = computed(() => sectionsByKind(visible.value))
const selected = computed(() => (selectedId.value ? store.connectionById(selectedId.value) : null))
const detail = computed(() => (selected.value ? store.detailOf(selected.value.id) : null))
const notFound = computed(() => store.loaded && selectedId.value !== null && selected.value === null)
const showSkeleton = computed(() => store.loading && !store.loaded)
const filtered = computed(() => activeFilterCount(filter.value) > 0)
const companies = computed(() => consoleStore.companies.map((company) => ({ id: company.id, name: company.name })))
const newConnectionCompanies = computed(() => (filter.value.companyId ? [filter.value.companyId] : []))
const backLabel = computed(() =>
  returnTo.value
    ? returnLabelOf(returnTo.value, new Map(consoleStore.projects.map((project) => [project.id as string, project.title])))
    : null
)
const filteredCompanyName = computed(
  () => companies.value.find((company) => company.id === filter.value.companyId)?.name ?? null
)
const filterSummary = computed(() =>
  [
    filter.value.companyId ? `company ${companies.value.find((company) => company.id === filter.value.companyId)?.name ?? 'unknown'}` : null,
    filter.value.kind ? `kind ${CONNECTION_KIND_LABEL[filter.value.kind]}` : null,
    filter.value.status ? `status ${CONNECTION_STATUS_LABEL[filter.value.status]}` : null
  ]
    .filter((part): part is string => part !== null)
    .join(', ')
)

async function load(): Promise<void> {
  loadFailed.value = false
  const read = await run(async () => {
    await store.load()
    return true
  })
  if (read === null) loadFailed.value = true
}

onMounted(load)

useConnectionStream(async () => {
  const before = store.connections
  await load()
  announcement.value = changeAnnouncement(before, store.connections)
  await detailPane.value?.refresh()
})

function openSheet(connection: Connection | null, kind: ConnectionKind | null): void {
  sheetFocus.remember()
  sheet.value = { connection, kind }
}

function closeSheet(): void {
  sheet.value = null
  sheetFocus.restore()
}

async function focusRow(id: ConnectionId | null): Promise<void> {
  await nextTick()
  const row = id ? document.querySelector<HTMLElement>(`[data-connection-id="${id}"]`) : null
  row?.focus()
}

async function saved(connection: Connection): Promise<void> {
  await select(connection.id)
  await nextTick()
  await nextTick()
  if (!connection.credentialSetAt) detailPane.value?.focusCredential()
}

function openCompanies(): void {
  dialogFocus.remember()
  editingCompanies.value = true
}

function closeCompanies(): void {
  editingCompanies.value = false
  dialogFocus.restore()
}

function askDelete(): void {
  dialogFocus.remember()
  deleting.value = selected.value
}

function keepConnection(): void {
  deleting.value = null
  dialogFocus.restore()
}

async function confirmDelete(): Promise<void> {
  const connection = deleting.value
  if (!connection) return
  const order = sections.value.flatMap((section) => section.connections.map((known) => known.id))
  const at = order.indexOf(connection.id)
  const next = order[at + 1] ?? order[at - 1] ?? null
  deleting.value = null
  const removed = await run(() => store.remove(connection), `Deleted ${connection.title}`)
  if (removed === null) return
  await select(next)
  await focusRow(next)
}

async function backToList(): Promise<void> {
  const leaving = selectedId.value
  await select(null)
  await focusRow(leaving)
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col overflow-hidden bg-canvas">
    <AppCatalogHeader title="Connections" compact>
      <template #actions>
        <AppButton variant="primary" size="sm" class="max-sm:min-h-11" data-testid="new-connection" @click="openSheet(null, null)"
          >New connection</AppButton
        >
      </template>
    </AppCatalogHeader>

    <p class="sr-only" role="status" aria-live="polite" data-testid="connections-live">{{ announcement }}</p>

    <RouterLink
      v-if="returnTo && backLabel"
      :to="returnTo"
      class="focus-ring mx-4 mt-3 inline-flex min-h-11 w-fit items-center gap-1.5 rounded-key text-small text-text-muted hover:text-text"
      data-testid="back-link"
    >
      <span aria-hidden="true">←</span> Back to {{ backLabel }}
    </RouterLink>

    <ConnectionFilterBar
      :class="selectedId ? 'max-[959px]:hidden' : ''"
      :filter="filter"
      :companies="companies"
      :hide-clear="!sections.length"
      @change="setFilter"
      @clear="clearFilters"
    />

    <div class="grid min-h-0 flex-1 border-t border-border min-[960px]:grid-cols-[340px_minmax(0,1fr)]">
      <div
        class="min-h-0 overflow-y-auto border-border min-[960px]:border-r"
        :class="selectedId ? 'hidden min-[960px]:block' : ''"
        data-testid="connections-list-pane"
      >
        <div v-if="showSkeleton" class="flex flex-col gap-2 p-4" aria-hidden="true" data-testid="connections-loading">
          <div v-for="row in 4" :key="row" class="skeleton h-14 rounded-control" />
        </div>

        <AppEmptyState
          v-else-if="loadFailed && !store.loaded"
          class="m-4"
          title="Connections did not load"
          description="The list could not be read. Check that Rekall is running, then try again."
          data-testid="connections-error"
        >
          <AppButton size="sm" variant="secondary" data-testid="retry-load" @click="load">Retry</AppButton>
        </AppEmptyState>

        <ConnectionEmptyState
          v-else-if="!sections.length"
          class="m-4"
          :filtered="filtered && store.connections.length > 0"
          :filter-summary="filterSummary"
          :company-name="filteredCompanyName"
          @start="openSheet(null, $event)"
          @create="openSheet(null, null)"
          @clear="clearFilters"
        />

        <ConnectionList v-else :sections="sections" :selected-id="selectedId" />
      </div>

      <div class="min-h-0 overflow-y-auto" :class="selectedId ? '' : 'hidden min-[960px]:block'" data-testid="connections-detail-pane">
        <ConnectionDetail
          v-if="selected"
          ref="detailPane"
          :key="selected.id"
          :connection="selected"
          @edit="openSheet(selected, null)"
          @delete="askDelete"
          @back="backToList"
          @edit-companies="openCompanies"
        />
        <AppEmptyState
          v-else-if="notFound"
          class="m-5"
          title="This connection no longer exists"
          description="It was deleted, possibly from another window."
          data-testid="connection-not-found"
        >
          <AppButton size="sm" variant="secondary" @click="select(null)">Back to connections</AppButton>
        </AppEmptyState>
        <p v-else-if="store.loaded && sections.length" class="p-8 text-small text-text-subtle" data-testid="nothing-selected">
          Select a connection to see its credential and who it is shared with.
        </p>
      </div>
    </div>

    <Transition name="slide-over">
      <ConnectionSheet
        v-if="sheet"
        :connection="sheet.connection"
        :initial-kind="sheet.kind"
        :initial-company-ids="newConnectionCompanies"
        :companies="companies"
        @close="closeSheet"
        @saved="saved"
        @edit-companies="closeSheet(); openCompanies()"
      />
    </Transition>

    <Transition name="dialog">
      <ConnectionCompaniesDialog v-if="editingCompanies && detail" :detail="detail" :companies="companies" @close="closeCompanies" />
    </Transition>

    <Transition name="dialog">
      <ConnectionDeleteDialog v-if="deleting" :connection="deleting" :detail="store.detailOf(deleting.id)" @cancel="keepConnection" @confirm="confirmDelete" />
    </Transition>
  </div>
</template>
