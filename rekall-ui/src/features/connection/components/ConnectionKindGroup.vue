<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { endpointOf } from '@/features/connection/lib/connection-endpoint'
import {
  CONNECTION_KIND_LABEL,
  CONNECTION_KIND_NOUN,
  type Connection,
  type ConnectionKind
} from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'

/** One kind's connections on the company screen, under its logo, with a way to add another of that kind. */
const props = defineProps<{
  kind: ConnectionKind
  connections: readonly Connection[]
  selectedId: ConnectionId | null
}>()
const emit = defineEmits<{ select: [id: ConnectionId]; add: [] }>()

const members = computed(() => props.connections.filter((connection) => connection.kind === props.kind))
const missing = computed(() => members.value.filter((connection) => !connection.credentialSetAt).length)
const headingId = computed(() => `connection-group-${props.kind.toLowerCase()}`)
</script>

<template>
  <section
    class="overflow-hidden rounded-card border border-border bg-surface"
    :aria-labelledby="headingId"
    :data-testid="`connection-group-${kind}`"
  >
    <header class="flex items-center gap-2.5 border-b border-border px-4 py-2.5">
      <ConnectionKindIcon :kind="kind" :size="18" />
      <h2 :id="headingId" class="text-body font-semibold text-text">{{ CONNECTION_KIND_LABEL[kind] }}</h2>
      <span
        class="font-mono text-caption tabular-nums text-text-subtle"
        data-testid="connection-group-count"
        >{{ members.length }}</span
      >
      <AppBadge v-if="missing" tone="warn" dot>{{ missing }} without a credential</AppBadge>
      <span class="flex-1" />
      <AppButton
        size="sm"
        variant="ghost"
        :aria-label="`Add a ${CONNECTION_KIND_NOUN[kind]}`"
        :data-testid="`add-connection-${kind}`"
        @click="emit('add')"
        >+ Add</AppButton
      >
    </header>

    <ul v-if="members.length" class="divide-y divide-border" data-testid="connection-list">
      <li v-for="connection in members" :key="connection.id">
        <button
          class="focus-ring flex min-h-14 w-full cursor-pointer items-center gap-3 px-4 py-2.5 text-left transition-colors hover:bg-surface-raised"
          :class="
            connection.id === selectedId ? 'bg-surface-raised shadow-[inset_2px_0_0_var(--color-accent)]' : ''
          "
          :aria-current="connection.id === selectedId"
          data-testid="connection-row"
          @click="emit('select', connection.id)"
        >
          <span class="min-w-0 flex-1">
            <span class="block truncate text-body font-semibold text-text">{{ connection.title }}</span>
            <span class="flex min-w-0 items-baseline gap-1.5 font-mono text-caption">
              <span class="shrink-0 text-anchor/80">{{ connection.label }}</span>
              <span class="truncate text-text-subtle">{{ endpointOf(connection) }}</span>
            </span>
          </span>
          <span class="flex shrink-0 flex-col items-end gap-1">
            <AppBadge :tone="connection.credentialSetAt ? 'safe' : 'warn'" dot>
              {{ connection.credentialSetAt ? 'Credential' : 'No credential' }}
            </AppBadge>
            <span class="text-caption text-text-subtle">
              <span class="font-mono tabular-nums text-text-muted">{{ connection.projectCount }}</span>
              project{{ connection.projectCount === 1 ? '' : 's' }}
            </span>
          </span>
        </button>
      </li>
    </ul>
    <p v-else class="px-4 py-3 text-caption text-text-subtle" data-testid="connection-group-empty">
      No {{ CONNECTION_KIND_NOUN[kind] }} yet.
    </p>
  </section>
</template>
