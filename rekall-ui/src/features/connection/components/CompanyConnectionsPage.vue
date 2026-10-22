<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import ConnectionDetailPane from '@/features/connection/components/ConnectionDetailPane.vue'
import ConnectionDialog from '@/features/connection/components/ConnectionDialog.vue'
import ConnectionKindGroup from '@/features/connection/components/ConnectionKindGroup.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConnectionStream } from '@/features/connection/composables/useConnectionStream'
import { CONNECTION_KINDS } from '@/features/connection/model/connection'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Connection, ConnectionKind } from '@/features/connection/model/connection'
import { asCompanyId, type ConnectionId } from '@/model/branded'

/** A company's connections: the GitLab instances, clusters and Jira sites its projects reach, managed in one place. */
const props = defineProps<{ id: string }>()

const consoleStore = useConsoleStore()
const store = useConnectionsStore()
const { run } = useAsyncAction()

const companyId = computed(() => asCompanyId(props.id))
const company = computed(
  () => consoleStore.companies.find((candidate) => candidate.id === companyId.value) ?? null
)
const connections = computed(() => store.connectionsOf(companyId.value))
const selectedId = ref<ConnectionId | null>(null)
const selected = computed(
  () => connections.value.find((connection) => connection.id === selectedId.value) ?? null
)
const dialog = ref<{ connection: Connection | null; kind: ConnectionKind | null } | null>(null)
const deleting = ref<Connection | null>(null)
const detailPane = ref<InstanceType<typeof ConnectionDetailPane> | null>(null)

async function load(): Promise<void> {
  await run(() => store.load(companyId.value))
  if (!selected.value) selectedId.value = connections.value[0]?.id ?? null
}

watch(companyId, load, { immediate: true })

useConnectionStream(async (event) => {
  if (event.companyId !== companyId.value) return
  await load()
  await detailPane.value?.refresh()
})

function blastOf(connection: Connection): string {
  const projects = `${connection.projectCount} project${connection.projectCount === 1 ? '' : 's'} lose${connection.projectCount === 1 ? 's' : ''} it`
  return `${projects} · its stored credential is removed from this machine`
}

async function confirmDelete(): Promise<void> {
  const connection = deleting.value
  if (!connection) return
  deleting.value = null
  await run(() => store.remove(connection), `Deleted ${connection.title}`)
  if (selectedId.value === connection.id) selectedId.value = connections.value[0]?.id ?? null
}
</script>

<template>
  <div class="flex-1 min-h-0 overflow-y-auto bg-canvas">
    <AppCatalogHeader title="Connections" :parent="company?.name ?? null">
      <template #actions>
        <AppButton
          variant="primary"
          size="sm"
          data-testid="new-connection"
          @click="dialog = { connection: null, kind: null }"
          >+ New connection</AppButton
        >
      </template>
    </AppCatalogHeader>

    <div class="mx-auto max-w-[1240px] px-8 py-6">
      <div
        v-if="store.isLoading(companyId) && !store.isLoaded(companyId)"
        class="flex flex-col gap-2"
        aria-hidden="true"
      >
        <div v-for="row in 3" :key="row" class="skeleton h-[64px] rounded-control" />
      </div>

      <AppEmptyState
        v-else-if="!connections.length"
        title="No connection yet"
        description="A connection is a GitLab instance, a Kubernetes cluster or a Jira site this company's projects reach. Its credential stays on this machine and never reaches a session."
      >
        <AppButton variant="secondary" size="sm" @click="dialog = { connection: null, kind: null }"
          >+ New connection</AppButton
        >
      </AppEmptyState>

      <div v-else class="grid gap-5 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
        <div class="flex h-fit flex-col gap-4">
          <ConnectionKindGroup
            v-for="kind in CONNECTION_KINDS"
            :key="kind"
            :kind="kind"
            :connections="connections"
            :selected-id="selectedId"
            @select="selectedId = $event"
            @add="dialog = { connection: null, kind }"
          />
        </div>

        <ConnectionDetailPane
          v-if="selected"
          ref="detailPane"
          :connection="selected"
          @edit="dialog = { connection: selected, kind: null }"
          @delete="deleting = selected"
        />
      </div>
    </div>

    <Transition name="dialog">
      <ConnectionDialog
        v-if="dialog"
        :company-id="companyId"
        :connection="dialog.connection"
        :initial-kind="dialog.kind"
        @close="dialog = null"
        @saved="selectedId = $event.id"
      />
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        :title="`Delete ${deleting.title}?`"
        body="Sessions on the projects that use it can no longer reach it."
        :blast="blastOf(deleting)"
        confirm-label="Delete connection"
        @cancel="deleting = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </div>
</template>
