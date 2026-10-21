<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import ConnectionDetailPane from '@/features/connection/components/ConnectionDetailPane.vue'
import ConnectionDialog from '@/features/connection/components/ConnectionDialog.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConnectionStream } from '@/features/connection/composables/useConnectionStream'
import { CONNECTION_KIND_LABEL } from '@/features/connection/model/connection'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Connection } from '@/features/connection/model/connection'
import { asCompanyId, type ConnectionId } from '@/model/branded'

/** A company's connections: the GitLab instances and clusters its projects reach, managed in one place. */
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
const dialog = ref<{ connection: Connection | null } | null>(null)
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
          @click="dialog = { connection: null }"
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
        description="A connection is a GitLab instance or a Kubernetes cluster this company's projects reach. Its credential stays on this machine and never reaches a session."
      >
        <AppButton variant="secondary" size="sm" @click="dialog = { connection: null }"
          >+ New connection</AppButton
        >
      </AppEmptyState>

      <div v-else class="grid gap-5 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
        <ul
          class="h-fit overflow-hidden rounded-card border border-border bg-surface"
          data-testid="connection-list"
        >
          <li
            v-for="(connection, index) in connections"
            :key="connection.id"
            :class="index > 0 ? 'border-t border-border' : ''"
          >
            <button
              class="focus-ring flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-surface-raised"
              :class="connection.id === selectedId ? 'bg-surface-raised' : ''"
              :aria-current="connection.id === selectedId"
              data-testid="connection-row"
              @click="selectedId = connection.id"
            >
              <span
                class="grid size-9 shrink-0 place-items-center rounded-card border border-border-strong font-mono text-caption font-semibold text-accent"
                aria-hidden="true"
              >
                {{ connection.kind === 'GITLAB' ? 'GL' : 'K8s' }}
              </span>
              <span class="min-w-0 flex-1">
                <span class="block truncate text-body font-semibold text-text">{{ connection.title }}</span>
                <span class="block truncate font-mono text-caption text-text-subtle">
                  {{ connection.label
                  }}<template v-if="connection.baseUrl"> · {{ connection.baseUrl }}</template>
                </span>
              </span>
              <span class="flex shrink-0 flex-col items-end gap-1">
                <AppBadge :tone="connection.credentialSetAt ? 'safe' : 'warn'" dot>
                  {{ connection.credentialSetAt ? 'Credential' : 'No credential' }}
                </AppBadge>
                <span class="text-caption text-text-subtle">
                  <span class="font-mono tabular-nums text-text-muted">{{ connection.projectCount }}</span>
                  project{{ connection.projectCount === 1 ? '' : 's' }} ·
                  {{ CONNECTION_KIND_LABEL[connection.kind] }}
                </span>
              </span>
            </button>
          </li>
        </ul>

        <ConnectionDetailPane
          v-if="selected"
          ref="detailPane"
          :connection="selected"
          @edit="dialog = { connection: selected }"
          @delete="deleting = selected"
        />
      </div>
    </div>

    <Transition name="dialog">
      <ConnectionDialog
        v-if="dialog"
        :company-id="companyId"
        :connection="dialog.connection"
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
