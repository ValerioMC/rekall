<script setup lang="ts">
import ConnectionKindGroup from '@/features/connection/components/ConnectionKindGroup.vue'
import type { KindSection } from '@/features/connection/lib/connection-filter'
import type { ConnectionId } from '@/model/branded'

/** The hub's list: a link per connection, grouped by kind. Up and Down move between the links. */
defineProps<{ sections: readonly KindSection[]; selectedId: ConnectionId | null }>()

function moveFocus(event: KeyboardEvent): void {
  if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return
  const rows = Array.from((event.currentTarget as HTMLElement).querySelectorAll<HTMLElement>('[data-connection-row]'))
  const at = rows.indexOf(document.activeElement as HTMLElement)
  if (at === -1) return
  const next = rows[at + (event.key === 'ArrowDown' ? 1 : -1)]
  if (!next) return
  event.preventDefault()
  next.focus()
}
</script>

<template>
  <nav aria-label="Connections" data-testid="connection-list" @keydown="moveFocus">
    <ConnectionKindGroup
      v-for="section in sections"
      :key="section.kind"
      :kind="section.kind"
      :connections="section.connections"
      :selected-id="selectedId"
    />
  </nav>
</template>
