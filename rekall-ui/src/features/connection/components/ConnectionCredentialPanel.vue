<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionCredentialField from '@/features/connection/components/ConnectionCredentialField.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { credentialNoun } from '@/features/connection/lib/credential-draft'
import { relativeTime } from '@/common/format/relative-time'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection } from '@/features/connection/model/connection'

/** A connection's credential: whether one is stored, adding, replacing or removing it, and asking the server whether it still works. */
const props = defineProps<{ connection: Connection }>()

const store = useConnectionsStore()
const { run, isRunning: checking } = useAsyncAction()
const root = ref<HTMLElement | null>(null)

const check = computed(() => store.checkOf(props.connection.id))
const noun = computed(() => credentialNoun(props.connection.kind))

async function runCheck(): Promise<void> {
  await run(() => store.check(props.connection.id))
}

function focusPrimary(): void {
  root.value?.querySelector<HTMLElement>('[data-testid="replace-credential"]')?.focus()
}

defineExpose({ focusPrimary })
</script>

<template>
  <section ref="root" aria-labelledby="connection-credential-heading" class="flex flex-col gap-3" data-testid="credential-panel">
    <h3 id="connection-credential-heading" class="text-body font-semibold text-text">Credential</h3>
    <ConnectionCredentialField :connection="connection" />
    <p v-if="!connection.credentialSetAt" class="text-small text-text-muted" data-testid="credential-none-note">
      No {{ noun }} stored. Sessions cannot reach {{ connection.title }} until one is added.
    </p>
    <div v-else class="flex flex-wrap items-center gap-3">
      <AppButton
        size="sm"
        variant="secondary"
        class="max-sm:min-h-11"
        :loading="checking"
        data-testid="check-connection"
        @click="runCheck"
      >
        Check
      </AppButton>
      <p
        v-if="check"
        class="min-w-0 flex-1 text-small"
        :class="check.ok ? 'text-safe' : 'text-danger'"
        role="status"
        data-testid="check-result"
      >
        {{ check.ok ? 'Works' : 'Check failed' }}: {{ check.detail }}
        <span class="text-caption text-text-subtle">{{ relativeTime(check.checkedAt) }}</span>
      </p>
      <p v-else class="text-caption text-text-subtle">Asks the server whether the stored credential still works.</p>
    </div>
  </section>
</template>
