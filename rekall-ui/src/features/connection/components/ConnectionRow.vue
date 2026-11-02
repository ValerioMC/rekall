<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import AppBadge from '@/components/ui/AppBadge.vue'
import ConnectionCredentialState from '@/features/connection/components/ConnectionCredentialState.vue'
import { plural } from '@/features/connection/lib/connection-impact'
import { scopeOf } from '@/features/connection/lib/connection-scope'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection } from '@/features/connection/model/connection'

/** One connection in the hub's list, a link to its detail: title, what it covers, its credential and how widely it is shared. */
const props = defineProps<{ connection: Connection; selected: boolean }>()

const route = useRoute()
const store = useConnectionsStore()

const target = computed(() => ({ name: 'connections', params: { id: props.connection.id }, query: route.query }))
const checkFailed = computed(() => store.checkOf(props.connection.id)?.ok === false)
const sharing = computed(
  () => `${plural(props.connection.companyIds.length, 'company', 'companies')} · ${plural(props.connection.projectCount, 'project')}`
)
</script>

<template>
  <RouterLink v-slot="{ href, navigate }" :to="target" custom>
    <a
      :href="href"
      class="focus-ring flex min-h-14 w-full items-center gap-3 px-4 py-2.5 text-left transition-colors hover:bg-surface-raised max-sm:min-h-16"
      :class="selected ? 'bg-surface-raised shadow-[inset_2px_0_0_var(--color-accent)]' : ''"
      :aria-current="selected ? 'true' : undefined"
      :data-connection-id="connection.id"
      data-connection-row
      data-testid="connection-row"
      @click="navigate"
    >
      <span class="min-w-0 flex-1">
        <span class="block truncate text-body font-semibold text-text">{{ connection.title }}</span>
        <span class="block truncate font-mono text-caption text-text-subtle" data-testid="connection-scope">{{
          scopeOf(connection)
        }}</span>
        <AppBadge v-if="!connection.companyIds.length" tone="warn" class="mt-1" data-testid="connection-unshared"
          >Not shared with any company</AppBadge
        >
      </span>
      <span class="flex shrink-0 flex-col items-end gap-1">
        <ConnectionCredentialState :credential-set-at="connection.credentialSetAt" :check-failed="checkFailed" />
        <span class="text-caption text-text-subtle" data-testid="connection-sharing">{{ sharing }}</span>
      </span>
    </a>
  </RouterLink>
</template>
