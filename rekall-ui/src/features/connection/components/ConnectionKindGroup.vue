<script setup lang="ts">
import { computed } from 'vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import ConnectionRow from '@/features/connection/components/ConnectionRow.vue'
import { CONNECTION_KIND_LABEL, type Connection, type ConnectionKind } from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'

/** One kind's connections in the hub's list, under its logo and name, with how many there are. */
const props = defineProps<{ kind: ConnectionKind; connections: readonly Connection[]; selectedId: ConnectionId | null }>()

const headingId = computed(() => `connection-group-${props.kind.toLowerCase()}`)
</script>

<template>
  <section :aria-labelledby="headingId" :data-testid="`connection-group-${kind}`">
    <h2 :id="headingId" class="flex items-center gap-2.5 px-4 pb-1.5 pt-4 text-small font-semibold text-text">
      <span aria-hidden="true"><ConnectionKindIcon :kind="kind" :size="16" /></span>
      <span data-testid="connection-group-name">{{ CONNECTION_KIND_LABEL[kind] }}</span>
      <span class="font-mono text-caption font-normal tabular-nums text-text-subtle" data-testid="connection-group-count"
        >({{ connections.length }})</span
      >
    </h2>
    <ul class="divide-y divide-border border-y border-border">
      <li v-for="connection in connections" :key="connection.id">
        <ConnectionRow :connection="connection" :selected="connection.id === selectedId" />
      </li>
    </ul>
  </section>
</template>
