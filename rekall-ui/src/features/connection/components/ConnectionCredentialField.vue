<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionCredentialState from '@/features/connection/components/ConnectionCredentialState.vue'
import ConnectionCredentialInput from '@/features/connection/components/ConnectionCredentialInput.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { credentialNoun, emptyDraft, isComplete } from '@/features/connection/lib/credential-draft'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection, CredentialDraft } from '@/features/connection/model/connection'

const props = defineProps<{ connection: Connection }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()
const editing = ref(false)
const draft = ref<CredentialDraft>(emptyDraft(props.connection.kind))

const noun = computed(() => credentialNoun(props.connection.kind))

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
    `Removed the ${noun.value} of ${props.connection.title}`
  )
}
</script>

<template>
  <div class="flex flex-col gap-2" data-testid="credential-field">
    <div class="flex items-center gap-2">
      <ConnectionCredentialState
        :credential-set-at="connection.credentialSetAt"
        detailed
        :data-testid="connection.credentialSetAt ? 'credential-set' : 'credential-missing'"
      />
      <span class="flex-1" />
      <template v-if="!editing">
        <AppButton
          size="sm"
          :variant="connection.credentialSetAt ? 'secondary' : 'primary'"
          class="max-sm:min-h-11"
          data-testid="replace-credential"
          @click="start"
        >
          {{ connection.credentialSetAt ? 'Replace' : 'Add' }} {{ noun }}
        </AppButton>
        <AppButton
          v-if="connection.credentialSetAt"
          size="sm"
          variant="danger-quiet"
          class="max-sm:min-h-11"
          :loading="isRunning"
          data-testid="clear-credential"
          @click="clear"
        >
          Remove
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
    <p v-if="connection.credentialSetAt" class="text-caption text-text-subtle">
      Kept in this machine's credential store and never shown again, here or to a session.
    </p>
  </div>
</template>
