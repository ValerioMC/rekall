<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionAccessLog from '@/features/connection/components/ConnectionAccessLog.vue'
import ConnectionCredentialPanel from '@/features/connection/components/ConnectionCredentialPanel.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import ConnectionSharingPanel from '@/features/connection/components/ConnectionSharingPanel.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { ADDRESS_TERM, ROOT_PATH_FIELD } from '@/features/connection/lib/connection-fields'
import { CONNECTION_KIND_LABEL } from '@/features/connection/model/connection'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection } from '@/features/connection/model/connection'

/** One connection in full: where it points, its credential, who it is shared with and who has asked for the credential. */
const props = defineProps<{ connection: Connection }>()
const emit = defineEmits<{ edit: []; delete: []; back: []; editCompanies: [] }>()

const store = useConnectionsStore()
const { run } = useAsyncAction()
const credentialPanel = ref<InstanceType<typeof ConnectionCredentialPanel> | null>(null)
const accessLog = ref<InstanceType<typeof ConnectionAccessLog> | null>(null)
const logOpened = ref(false)

const detail = computed(() => store.detailOf(props.connection.id))

async function loadDetail(): Promise<void> {
  await run(() => store.loadDetail(props.connection.id))
}

onMounted(loadDetail)
watch(() => props.connection.id, loadDetail)
watch(() => props.connection.updatedAt, loadDetail)

defineExpose({
  refresh: async () => {
    await loadDetail()
    await accessLog.value?.reload()
  },
  focusCredential: () => credentialPanel.value?.focusPrimary()
})
</script>

<template>
  <section class="flex flex-col gap-6 p-5" :aria-label="connection.title" data-testid="connection-detail">
    <button
      type="button"
      class="focus-ring -ml-1 inline-flex min-h-11 w-fit items-center gap-1.5 rounded-key px-1 text-small text-text-muted hover:text-text min-[960px]:hidden"
      data-testid="detail-back"
      @click="emit('back')"
    >
      <span aria-hidden="true">←</span> Connections
    </button>

    <header class="flex flex-col gap-3">
      <div class="flex items-start gap-3">
        <span class="grid size-10 shrink-0 place-items-center rounded-card border border-border-strong bg-surface-raised">
          <ConnectionKindIcon :kind="connection.kind" :size="22" />
        </span>
        <div class="min-w-0 flex-1">
          <h2 class="truncate text-title font-semibold text-text" data-testid="detail-title">{{ connection.title }}</h2>
          <p class="text-caption text-text-subtle" data-testid="connection-detail-kind">{{ CONNECTION_KIND_LABEL[connection.kind] }}</p>
        </div>
        <AppButton size="sm" variant="secondary" class="max-sm:min-h-11" data-testid="edit-connection" @click="emit('edit')">Edit</AppButton>
        <AppButton size="sm" variant="danger-quiet" class="ml-3 max-sm:hidden" data-testid="delete-connection" @click="emit('delete')"
          >Delete</AppButton
        >
        <details class="relative sm:hidden">
          <summary class="focus-ring grid size-11 cursor-pointer list-none place-items-center rounded-control text-text-muted hover:bg-surface-raised" aria-label="More actions">
            <span aria-hidden="true">⋯</span>
          </summary>
          <div class="absolute right-0 z-10 mt-1 rounded-control border border-border-strong bg-surface p-1 shadow-modal">
            <AppButton variant="danger-quiet" class="min-h-11" data-testid="delete-connection-menu" @click="emit('delete')">Delete</AppButton>
          </div>
        </details>
      </div>

      <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-small">
        <template v-if="connection.baseUrl">
          <dt class="text-text-subtle">{{ ADDRESS_TERM[connection.kind] }}</dt>
          <dd class="truncate font-mono text-text">{{ connection.baseUrl }}</dd>
        </template>
        <template v-if="connection.rootPath && ROOT_PATH_FIELD[connection.kind]">
          <dt class="text-text-subtle">{{ ROOT_PATH_FIELD[connection.kind]?.title }}</dt>
          <dd class="truncate font-mono text-text" data-testid="connection-root-path-value">{{ connection.rootPath }}</dd>
        </template>
        <template v-if="connection.proxyUrl">
          <dt class="text-text-subtle">Proxy</dt>
          <dd class="truncate font-mono text-text" data-testid="connection-proxy-url">{{ connection.proxyUrl }}</dd>
        </template>
        <template v-if="connection.kubeContext">
          <dt class="text-text-subtle">Context</dt>
          <dd class="font-mono text-text">{{ connection.kubeContext }}</dd>
        </template>
      </dl>

      <details>
        <summary class="focus-ring w-fit cursor-pointer rounded-key text-caption text-text-subtle hover:text-text">Advanced</summary>
        <p class="mt-1.5 text-caption text-text-subtle">
          <span class="font-mono text-anchor/80" data-testid="connection-label-value">{{ connection.label }}</span>
          is the short name sessions use: kubectl --context, rekall-gitlab, rekall-github and rekall-jira --connection.
        </p>
      </details>
    </header>

    <ConnectionCredentialPanel ref="credentialPanel" :connection="connection" />

    <ConnectionSharingPanel :detail="detail" @edit-companies="emit('editCompanies')" />

    <details @toggle="logOpened = true">
      <summary class="focus-ring cursor-pointer rounded-key text-small font-semibold text-text">Access log</summary>
      <div class="mt-2">
        <ConnectionAccessLog v-if="logOpened" ref="accessLog" :connection-id="connection.id" />
      </div>
    </details>
  </section>
</template>
