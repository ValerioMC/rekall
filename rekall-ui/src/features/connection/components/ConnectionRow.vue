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
const unshared = computed(() => props.connection.companyIds.length === 0)
const sharing = computed(() => {
  const projects = plural(props.connection.projectCount, 'project')
  return unshared.value ? projects : `${plural(props.connection.companyIds.length, 'company', 'companies')} · ${projects}`
})
</script>

<template>
  <RouterLink v-slot="{ href, navigate }" :to="target" custom>
    <a
      :href="href"
      class="focus-ring flex min-h-14 w-full flex-wrap items-center gap-x-3 gap-y-1 rounded-control py-2.5 pl-4 pr-3 text-left transition-colors max-sm:min-h-16"
      :class="selected ? 'selected-row' : 'hover:bg-surface-raised'"
      :aria-current="selected ? 'true' : undefined"
      :data-connection-id="connection.id"
      data-connection-row
      data-testid="connection-row"
      @click="navigate"
    >
      <span class="min-w-0 flex-1">
        <span class="block truncate text-body font-semibold text-text">{{ connection.title }}</span>
        <span class="block line-clamp-2 break-all font-mono text-caption text-text-subtle" data-testid="connection-scope">{{
          scopeOf(connection)
        }}</span>
      </span>
      <span class="flex shrink-0 flex-col items-end gap-1">
        <ConnectionCredentialState :credential-set-at="connection.credentialSetAt" :check-failed="checkFailed" />
        <span class="text-small text-text-subtle" data-testid="connection-sharing">{{ sharing }}</span>
      </span>
      <AppBadge v-if="unshared" tone="warn" dot class="w-full" data-testid="connection-unshared">Not shared with any company</AppBadge>
    </a>
  </RouterLink>
</template>
