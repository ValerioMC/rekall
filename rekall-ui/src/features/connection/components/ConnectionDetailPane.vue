<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionAccessLog from '@/features/connection/components/ConnectionAccessLog.vue'
import ConnectionCredentialField from '@/features/connection/components/ConnectionCredentialField.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { CONNECTION_KIND_LABEL } from '@/features/connection/model/connection'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection } from '@/features/connection/model/connection'

/** One connection in full: its credential, whether it still works, who uses it and who asked for it. */
const props = defineProps<{ connection: Connection }>()
const emit = defineEmits<{ edit: []; delete: [] }>()

const store = useConnectionsStore()
const { run } = useAsyncAction()
const checking = ref(false)
const accessLog = ref<InstanceType<typeof ConnectionAccessLog> | null>(null)

const detail = computed(() => store.detailOf(props.connection.id))
const check = computed(() => store.checkOf(props.connection.id))

async function loadDetail(): Promise<void> {
  await run(() => store.loadDetail(props.connection.id))
}

async function runCheck(): Promise<void> {
  checking.value = true
  await run(() => store.check(props.connection.id))
  checking.value = false
}

onMounted(loadDetail)
watch(() => props.connection.id, loadDetail)
watch(() => props.connection.updatedAt, loadDetail)

defineExpose({
  refresh: async () => {
    await loadDetail()
    await accessLog.value?.reload()
  }
})
</script>

<template>
  <section
    class="flex flex-col gap-5 rounded-card border border-border bg-surface p-5"
    data-testid="connection-detail"
  >
    <header class="flex items-start gap-3">
      <span
        class="grid size-10 shrink-0 place-items-center rounded-card border border-border-strong bg-surface-raised"
        data-testid="connection-detail-kind"
      >
        <ConnectionKindIcon :kind="connection.kind" :size="22" />
      </span>
      <span class="min-w-0 flex-1">
        <span class="block text-caption text-text-subtle">{{ CONNECTION_KIND_LABEL[connection.kind] }}</span>
        <span class="block truncate text-title font-semibold text-text">{{ connection.title }}</span>
        <span class="block font-mono text-caption text-anchor/80">{{ connection.label }}</span>
      </span>
      <AppButton size="sm" variant="ghost" data-testid="edit-connection" @click="emit('edit')"
        >Edit</AppButton
      >
      <AppButton size="sm" variant="danger-quiet" data-testid="delete-connection" @click="emit('delete')"
        >Delete</AppButton
      >
    </header>

    <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-small">
      <template v-if="connection.baseUrl">
        <dt class="text-text-subtle">{{ connection.kind === 'GITLAB' ? 'Instance' : 'Server' }}</dt>
        <dd class="truncate font-mono text-text">{{ connection.baseUrl }}</dd>
      </template>
      <template v-if="connection.kubeContext">
        <dt class="text-text-subtle">Context</dt>
        <dd class="font-mono text-text">{{ connection.kubeContext }}</dd>
      </template>
    </dl>

    <div>
      <p class="mb-2 eyebrow text-caption">Credential</p>
      <ConnectionCredentialField :connection="connection" />
    </div>

    <div>
      <p class="mb-2 eyebrow text-caption">Check</p>
      <div class="flex items-center gap-3">
        <AppButton
          size="sm"
          variant="secondary"
          :loading="checking"
          :disabled="!connection.credentialSetAt"
          data-testid="check-connection"
          @click="runCheck"
        >
          Check now
        </AppButton>
        <span
          v-if="check"
          class="min-w-0 flex-1 text-small"
          :class="check.ok ? 'text-safe' : 'text-danger'"
          data-testid="check-result"
        >
          {{ check.detail }}
          <span class="text-caption text-text-subtle">· {{ relativeTime(check.checkedAt) }}</span>
        </span>
        <span v-else class="text-caption text-text-subtle"
          >Asks the server whether the stored credential still works.</span
        >
      </div>
    </div>

    <div>
      <p class="mb-2 eyebrow text-caption">Used by</p>
      <p v-if="detail && !detail.projects.length" class="text-caption text-text-subtle">
        No project uses it yet. Add it from a project's page.
      </p>
      <ul v-else-if="detail" class="flex flex-col gap-1">
        <li
          v-for="project in detail.projects"
          :key="project.projectId"
          class="flex items-baseline gap-2 text-small"
        >
          <RouterLink
            :to="`/projects/${project.projectId}`"
            class="focus-ring rounded-key text-text hover:text-accent"
            >{{ project.title }}</RouterLink
          >
          <span class="font-mono text-caption text-text-subtle">project:{{ project.label }}</span>
          <span v-if="project.target" class="ml-auto font-mono text-caption text-text-muted">{{
            project.target
          }}</span>
        </li>
      </ul>
    </div>

    <div>
      <p class="mb-2 eyebrow text-caption">Access log</p>
      <ConnectionAccessLog ref="accessLog" :connection-id="connection.id" />
    </div>
  </section>
</template>
