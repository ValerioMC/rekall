<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
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
  <section class="flex max-w-[760px] flex-col gap-4 p-5" :aria-label="connection.title" data-testid="connection-detail">
    <button
      type="button"
      class="focus-ring -ml-1 inline-flex min-h-11 w-fit items-center gap-1.5 rounded-key px-1 text-small text-text-muted hover:text-text min-[960px]:hidden"
      data-testid="detail-back"
      @click="emit('back')"
    >
      <span aria-hidden="true">←</span> Connections
    </button>

    <header class="project-header-card flex flex-col gap-4 rounded-card p-4">
      <div class="flex items-start gap-3">
        <span class="grid size-11 shrink-0 place-items-center rounded-card border border-border-strong bg-canvas shadow-[var(--shadow-well)]">
          <ConnectionKindIcon :kind="connection.kind" :size="24" />
        </span>
        <div class="min-w-0 flex-1">
          <h2 class="truncate text-heading font-semibold tracking-[-0.01em] text-text" data-testid="detail-title">{{ connection.title }}</h2>
          <p class="mt-0.5 text-small text-text-subtle" data-testid="connection-detail-kind">{{ CONNECTION_KIND_LABEL[connection.kind] }}</p>
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

      <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 rounded-control border border-border bg-canvas/60 px-3 py-2.5 text-small shadow-[var(--shadow-well)] empty:hidden">
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

      <p class="flex flex-wrap items-center gap-x-2 gap-y-1 text-caption text-text-subtle">
        <span class="anchor-chip px-1.5 py-px text-caption" data-testid="connection-label-value">{{ connection.label }}</span>
        <span>is the short name sessions use: kubectl --context, rekall-gitlab, rekall-github and rekall-jira --connection.</span>
      </p>
    </header>

    <AppCard>
      <ConnectionCredentialPanel ref="credentialPanel" :connection="connection" />
    </AppCard>

    <AppCard>
      <ConnectionSharingPanel :detail="detail" @edit-companies="emit('editCompanies')" />
    </AppCard>

    <AppCard :padded="false">
      <details class="group" @toggle="logOpened = true">
        <summary
          class="focus-ring flex cursor-pointer list-none items-center gap-2 rounded-card px-5 py-3.5 text-body font-semibold text-text hover:bg-surface-hover/40 [&::-webkit-details-marker]:hidden"
        >
          <svg viewBox="0 0 24 24" class="size-3.5 text-text-subtle transition-transform group-open:rotate-90" fill="none" aria-hidden="true">
            <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
          </svg>
          Access log
        </summary>
        <div class="border-t border-border px-5 py-2">
          <ConnectionAccessLog v-if="logOpened" ref="accessLog" :connection-id="connection.id" />
        </div>
      </details>
    </AppCard>
  </section>
</template>
