<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { useCredentialAccessStore } from '@/features/connection/store/credentialAccess.store'
import { useConsoleStore } from '@/stores/console.store'
import type { TerminalConnection } from '@/features/claude/model/terminal'
import type { ProjectId, TerminalId } from '@/model/branded'

/**
 * What a running terminal can reach outside Rekall: a chip per connection, counting the times its
 * helpers were handed the credential and warning when one was refused or none is stored.
 */
const props = defineProps<{
  terminalId: TerminalId
  projectId: ProjectId
  connections: readonly TerminalConnection[]
}>()

const access = useCredentialAccessStore()
const consoleStore = useConsoleStore()

const companyId = computed(
  () => consoleStore.projects.find((project) => project.id === props.projectId)?.companyId ?? null
)

const chips = computed(() =>
  props.connections.map((connection) => {
    const tally = access.tallyOf(props.terminalId, connection.connectionId)
    const tone =
      !connection.credentialSet || (tally?.refused ?? 0) > 0
        ? 'warn'
        : (tally?.granted ?? 0) > 0
          ? 'used'
          : 'idle'
    return { connection, tally, tone }
  })
)

function describe(connection: TerminalConnection, granted: number, refused: number): string {
  if (!connection.credentialSet)
    return `${connection.label}: no credential is stored, so this session cannot reach it`
  const parts = [`${connection.label}: handed out ${granted} time${granted === 1 ? '' : 's'}`]
  if (refused) parts.push(`refused ${refused} time${refused === 1 ? '' : 's'}`)
  return parts.join(', ')
}

onMounted(access.listen)
</script>

<template>
  <span
    v-if="connections.length"
    class="inline-flex flex-wrap items-center gap-1"
    data-testid="terminal-connections"
  >
    <component
      :is="companyId ? RouterLink : 'span'"
      v-for="chip in chips"
      :key="`${chip.connection.label}-${chip.tally?.last ?? 0}`"
      :to="companyId ? `/companies/${companyId}/connections` : undefined"
      class="focus-ring inline-flex items-center gap-1 rounded-full px-2 py-0.5 font-mono text-micro ring-1 ring-inset"
      :class="{
        'bg-warn-soft text-warn ring-warn/25': chip.tone === 'warn',
        'connection-pulse bg-safe-soft text-safe ring-safe/25': chip.tone === 'used',
        'bg-surface-raised text-text-muted ring-border': chip.tone === 'idle'
      }"
      :title="describe(chip.connection, chip.tally?.granted ?? 0, chip.tally?.refused ?? 0)"
      :data-testid="`terminal-connection-${chip.connection.label}`"
      :data-tone="chip.tone"
    >
      <ConnectionKindIcon :kind="chip.connection.kind" :size="11" />
      {{ chip.connection.label }}
      <span v-if="chip.tally" class="tabular-nums"
        >·{{ chip.tally.granted
        }}<template v-if="chip.tally.refused">/{{ chip.tally.refused }}✕</template></span
      >
    </component>
  </span>
</template>

<style scoped>
/* Replayed on every access: the key changes with the tally, so the chip mounts again and pulses once. */
.connection-pulse {
  animation: connection-pulse var(--motion-slow, 600ms) ease-out 1;
}

@keyframes connection-pulse {
  from {
    box-shadow: 0 0 0 0 color-mix(in srgb, currentColor 45%, transparent);
  }
  to {
    box-shadow: 0 0 0 6px transparent;
  }
}

@media (prefers-reduced-motion: reduce) {
  .connection-pulse {
    animation: none;
  }
}
</style>
