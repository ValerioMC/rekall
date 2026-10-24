<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { DISCOVERY_OUTCOME_LABEL, relativeFolder } from '@/features/connection/lib/discovery-outcome'
import type { RepositoryDiscovery } from '@/features/connection/model/connection'

/**
 * What the last repository discovery found under the project's folder: each repository, the
 * binding that reaches it and the component made for it, or why it was left out.
 */
defineProps<{ discovery: RepositoryDiscovery }>()
const emit = defineEmits<{ dismiss: [] }>()
</script>

<template>
  <div class="mt-2 rounded-control border border-border p-2" data-testid="repository-discovery">
    <div class="mb-1 flex items-center gap-2">
      <span class="eyebrow min-w-0 flex-1 truncate text-caption">Repositories in {{ discovery.folder }}</span>
      <button
        class="focus-ring grid size-6 place-items-center rounded-key text-text-subtle hover:bg-surface-hover"
        aria-label="Hide the repositories found"
        data-testid="dismiss-discovery"
        @click="emit('dismiss')"
      >
        <CloseGlyph />
      </button>
    </div>
    <p v-if="!discovery.repositories.length" class="text-caption text-text-subtle">
      No git repository under this folder.
    </p>
    <ul v-else class="flex flex-col gap-1">
      <li
        v-for="repository in discovery.repositories"
        :key="repository.folder"
        class="flex flex-wrap items-baseline gap-x-2 text-caption"
        data-testid="discovered-repository"
      >
        <span class="font-mono text-text">{{ relativeFolder(discovery.folder, repository) }}</span>
        <span v-if="repository.label" class="font-mono text-anchor/80">{{ repository.label }}</span>
        <span v-if="repository.origin" class="min-w-0 truncate font-mono text-text-subtle">{{ repository.origin }}</span>
        <span class="ml-auto flex items-center gap-1">
          <AppBadge v-if="repository.component" tone="neutral">component {{ repository.component }}</AppBadge>
          <AppBadge :tone="repository.bindingId ? 'safe' : 'warn'" dot>{{
            DISCOVERY_OUTCOME_LABEL[repository.outcome]
          }}</AppBadge>
        </span>
      </li>
    </ul>
  </div>
</template>
