<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import ProjectConnectionRow from '@/features/connection/components/ProjectConnectionRow.vue'
import type { ConnectionTrunk } from '@/features/connection/lib/connection-trunks'
import type { ConnectionKind, ProjectConnection, ProjectConnectionInput } from '@/features/connection/model/connection'

/**
 * One company connection and the bindings this project hangs off it, drawn on a line in its
 * system's colour: the credential is set once, on the connection, and reaches every binding.
 */
defineProps<{ trunk: ConnectionTrunk }>()

const emit = defineEmits<{
  addTarget: []
  save: [binding: ProjectConnection, input: ProjectConnectionInput]
  unbind: [binding: ProjectConnection]
  importIssues: [binding: ProjectConnection]
}>()

const LINE: Readonly<Record<ConnectionKind, string>> = {
  GITLAB: 'color-mix(in srgb, var(--color-brand-gitlab) 55%, transparent)',
  KUBERNETES: 'color-mix(in srgb, var(--color-brand-kubernetes) 70%, transparent)',
  JIRA: 'color-mix(in srgb, var(--color-brand-jira) 60%, transparent)'
}
</script>

<template>
  <section class="trunk" :style="{ '--branch-line': LINE[trunk.kind] }" :data-testid="`trunk-${trunk.connectionId}`">
    <header class="flex min-h-9 flex-wrap items-center gap-x-3 gap-y-1">
      <span class="trunk-node grid size-[23px] shrink-0 place-items-center rounded-full" aria-hidden="true">
        <ConnectionKindIcon :kind="trunk.kind" :size="13" />
      </span>
      <h3 class="text-body font-semibold text-text">{{ trunk.title }}</h3>
      <span class="font-mono text-caption text-text-subtle">{{ trunk.endpoint }}</span>
      <AppBadge v-if="trunk.hasCredential" tone="safe" dot>Credential stored</AppBadge>
      <AppBadge v-else tone="warn" dot data-testid="project-connection-missing">No credential</AppBadge>
      <span class="ml-auto font-mono text-micro tabular-nums text-text-subtle">
        {{ trunk.bindings.length === 1 ? '1 binding' : `${trunk.bindings.length} bindings` }}
      </span>
      <AppButton
        size="sm"
        variant="ghost"
        :title="`Use ${trunk.title} once more, for another target`"
        data-testid="add-target"
        @click="emit('addTarget')"
        >+ Target</AppButton
      >
    </header>
    <ul class="branch-list">
      <li v-for="binding in trunk.bindings" :key="binding.id">
        <ProjectConnectionRow
          :binding="binding"
          @save="emit('save', binding, $event)"
          @unbind="emit('unbind', binding)"
          @import-issues="emit('importIssues', binding)"
        />
      </li>
    </ul>
  </section>
</template>

<style scoped>
/* The connection's node is where its line starts: the line leaves from under the logo. */
.trunk-node {
  border: 1px solid var(--branch-line);
  background: var(--color-surface-raised);
  box-shadow: 0 0 0 3px var(--color-surface);
}

.trunk .branch-list {
  margin-top: 2px;
}
</style>
