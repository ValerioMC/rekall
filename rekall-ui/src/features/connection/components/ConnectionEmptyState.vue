<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { KIND_ORDER } from '@/features/connection/lib/connection-order'
import { CONNECTION_KIND_LABEL, type ConnectionKind } from '@/features/connection/model/connection'

/**
 * Nothing to list. With no connection at all it explains what one is and starts one of each kind;
 * under filters it names them and offers to clear.
 */
const props = defineProps<{ filtered: boolean; filterSummary: string; companyName: string | null }>()
const emit = defineEmits<{ start: [kind: ConnectionKind]; create: []; clear: [] }>()

const filteredTitle = computed(() =>
  props.companyName ? `No connections are shared with ${props.companyName}` : 'No connection matches'
)
</script>

<template>
  <AppEmptyState
    v-if="filtered"
    :title="filteredTitle"
    :description="`Filtered by ${filterSummary}.`"
    data-testid="connections-empty-filtered"
  >
    <div class="flex flex-wrap justify-center gap-2">
      <AppButton v-if="companyName" variant="primary" size="sm" data-testid="empty-new-connection" @click="emit('create')"
        >New connection for {{ companyName }}</AppButton
      >
      <AppButton variant="secondary" size="sm" data-testid="empty-clear-filters" @click="emit('clear')">Clear filters</AppButton>
    </div>
  </AppEmptyState>
  <AppEmptyState
    v-else
    title="No connection yet"
    description="A connection is an account or server your projects reach: a GitLab instance, a GitHub account, a Kubernetes cluster, a Jira site or a Grafana instance. Its credential stays on this machine and is shared with the companies you choose."
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
