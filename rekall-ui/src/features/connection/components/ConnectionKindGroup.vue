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
    <h2 :id="headingId" class="section-label flex items-center gap-2.5 px-4 pb-2 pt-5">
      <span class="grid size-6 place-items-center rounded-control border border-border bg-surface-raised" aria-hidden="true">
        <ConnectionKindIcon :kind="kind" :size="13" />
      </span>
      <span data-testid="connection-group-name">{{ CONNECTION_KIND_LABEL[kind] }}</span>
      <span
        class="rounded-key bg-surface-raised px-1.5 py-px font-mono text-micro font-normal tabular-nums text-text-subtle"
        data-testid="connection-group-count"
        >({{ connections.length }})</span
      >
    </h2>
    <ul class="flex flex-col gap-1 px-2">
      <li v-for="connection in connections" :key="connection.id">
        <ConnectionRow :connection="connection" :selected="connection.id === selectedId" />
      </li>
    </ul>
  </section>
</template>
