<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { KIND_ORDER } from '@/features/connection/lib/connection-order'
import { CONNECTION_KIND_LABEL, type ConnectionKind } from '@/features/connection/model/connection'

/**
 * Nothing to list. With no connection at all it explains what one is and starts one of each kind;
 * under filters it names them and offers to clear.
 */
defineProps<{ filtered: boolean; filterSummary: string }>()
const emit = defineEmits<{ start: [kind: ConnectionKind]; clear: [] }>()
</script>

<template>
  <AppEmptyState
    v-if="filtered"
    title="No connection matches"
    :description="`Filtered by ${filterSummary}.`"
    data-testid="connections-empty-filtered"
  >
    <AppButton variant="secondary" size="sm" data-testid="empty-clear-filters" @click="emit('clear')">Clear filters</AppButton>
  </AppEmptyState>
  <AppEmptyState
    v-else
    title="No connection yet"
    description="A connection is an account or server your projects reach: a GitLab instance, a GitHub account, a Kubernetes cluster or a Jira site. Its credential stays on this machine and is shared with the companies you choose."
    data-testid="connections-empty"
  >
    <div class="flex flex-wrap justify-center gap-2">
      <AppButton v-for="kind in KIND_ORDER" :key="kind" size="sm" :data-testid="`start-${kind}`" @click="emit('start', kind)">
        <ConnectionKindIcon :kind="kind" :size="14" />
        {{ CONNECTION_KIND_LABEL[kind] }}
      </AppButton>
    </div>
  </AppEmptyState>
</template>
