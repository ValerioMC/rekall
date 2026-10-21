<script setup lang="ts">
import { computed, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionCredentialInput from '@/features/connection/components/ConnectionCredentialInput.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { emptyDraft, isComplete } from '@/features/connection/lib/credential-draft'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection, CredentialDraft } from '@/features/connection/model/connection'

const props = defineProps<{ connection: Connection }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()
const editing = ref(false)
const draft = ref<CredentialDraft>(emptyDraft(props.connection.kind))

const noun = computed(() => (props.connection.kind === 'GITLAB' ? 'token' : 'kubeconfig'))

function start(): void {
  draft.value = emptyDraft(props.connection.kind)
  editing.value = true
}

/** The typed credential is dropped whatever the answer, so it never outlives its one request. */
function stop(): void {
  draft.value = emptyDraft(props.connection.kind)
  editing.value = false
}

async function save(): Promise<void> {
  if (!isComplete(draft.value)) return
  const sending = draft.value
  draft.value = emptyDraft(props.connection.kind)
  const saved = await run(
    () => store.setCredential(props.connection.id, sending),
    `Stored the ${noun.value} of ${props.connection.title}`
  )
  if (saved) stop()
}

async function clear(): Promise<void> {
  await run(
    () => store.clearCredential(props.connection.id),
    `Cleared the ${noun.value} of ${props.connection.title}`
  )
}
</script>

<template>
  <div class="flex flex-col gap-2" data-testid="credential-field">
    <div class="flex items-center gap-2">
      <AppBadge v-if="connection.credentialSetAt" tone="safe" dot data-testid="credential-set">
        Stored {{ relativeTime(connection.credentialSetAt) }}
      </AppBadge>
      <AppBadge v-else tone="warn" dot data-testid="credential-missing">No {{ noun }} stored</AppBadge>
      <span class="flex-1" />
      <template v-if="!editing">
        <AppButton size="sm" variant="secondary" data-testid="replace-credential" @click="start">
          {{ connection.credentialSetAt ? 'Replace' : 'Set' }} {{ noun }}
        </AppButton>
        <AppButton
          v-if="connection.credentialSetAt"
          size="sm"
          variant="danger-quiet"
          :loading="isRunning"
          data-testid="clear-credential"
          @click="clear"
        >
          Clear
        </AppButton>
      </template>
    </div>
    <template v-if="editing">
      <ConnectionCredentialInput v-model="draft" />
      <div class="flex justify-end gap-2">
        <AppButton size="sm" variant="ghost" @click="stop">Cancel</AppButton>
        <AppButton
          size="sm"
          variant="primary"
          :disabled="!isComplete(draft)"
          :loading="isRunning"
          data-testid="save-credential"
          @click="save"
        >
          Store {{ noun }}
        </AppButton>
      </div>
    </template>
    <p class="text-caption text-text-subtle">
      Kept in this machine's credential store and never shown again, here or to a session.
    </p>
  </div>
</template>
