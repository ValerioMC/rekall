<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionCredentialState from '@/features/connection/components/ConnectionCredentialState.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { scopeOf } from '@/features/connection/lib/connection-scope'
import type { Connection } from '@/features/connection/model/connection'

/** One connection a project could bind: what decides the choice (its title, what it covers, whether it has a credential) and a way to use it. */
defineProps<{ connection: Connection }>()
const emit = defineEmits<{ use: [] }>()
</script>

<template>
  <div class="flex min-h-11 flex-wrap items-center gap-x-3 gap-y-1 py-1" data-testid="connection-option">
    <ConnectionKindIcon :kind="connection.kind" :size="14" />
    <span class="min-w-0 truncate text-small font-medium text-text">{{ connection.title }}</span>
    <span class="min-w-0 truncate font-mono text-caption text-text-subtle" data-testid="connection-option-scope">{{
      scopeOf(connection)
    }}</span>
    <ConnectionCredentialState :credential-set-at="connection.credentialSetAt" />
    <AppButton class="ml-auto max-sm:min-h-11" size="sm" variant="secondary" :data-testid="`use-connection-${connection.id}`" @click="emit('use')"
      >Use</AppButton
    >
  </div>
</template>
