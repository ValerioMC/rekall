<script setup lang="ts">
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionCredentialState from '@/features/connection/components/ConnectionCredentialState.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import ProjectConnectionRow from '@/features/connection/components/ProjectConnectionRow.vue'
import type { ConnectionTrunk } from '@/features/connection/lib/connection-trunks'
import type { ConnectionKind, ProjectConnection, ProjectConnectionInput } from '@/features/connection/model/connection'

/**
 * One connection and the bindings this project hangs off it, drawn on a line in its system's
 * colour: the credential is set once, on the connection, and reaches every binding. The state is
 * read-only here; the link goes to the connection in the Connections screen, where it is changed.
 */
defineProps<{ trunk: ConnectionTrunk; returnTo: string }>()

const emit = defineEmits<{
  addTarget: []
  save: [binding: ProjectConnection, input: ProjectConnectionInput]
  unbind: [binding: ProjectConnection]
  importIssues: [binding: ProjectConnection]
}>()

const LINE: Readonly<Record<ConnectionKind, string>> = {
  GITLAB: 'color-mix(in srgb, var(--color-brand-gitlab) 55%, transparent)',
  GITHUB: 'color-mix(in srgb, var(--color-text) 40%, transparent)',
  KUBERNETES: 'color-mix(in srgb, var(--color-brand-kubernetes) 70%, transparent)',
  JIRA: 'color-mix(in srgb, var(--color-brand-jira) 60%, transparent)',
  GRAFANA: 'color-mix(in srgb, var(--color-brand-grafana) 60%, transparent)'
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
      <ConnectionCredentialState
        :credential-set-at="trunk.credentialSetAt"
        :data-testid="trunk.credentialSetAt ? 'project-connection-stored' : 'project-connection-missing'"
      />
      <RouterLink
        :to="{ name: 'connections', params: { id: trunk.connectionId }, query: { returnTo } }"
        class="focus-ring rounded-key text-caption text-accent hover:underline"
        :aria-label="`Open ${trunk.title} in Connections`"
        data-testid="open-in-connections"
        >Open in Connections</RouterLink
      >
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
          :root-path="trunk.rootPath"
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
