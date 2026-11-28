<script setup lang="ts">
import { computed } from 'vue'
import { useExternalLink } from '@/composables/useExternalLink'
import { deliveryOf, type RequestState } from '@/features/project/lib/task-delivery'
import type { Task } from '@/features/catalog/model/catalog'

/** A task's issue and pull or merge request as links, the request with the state it is in. */
const props = defineProps<{ task: Task }>()

const REQUEST_TONE: Readonly<Record<RequestState, string>> = {
  open: 'text-text-muted',
  merged: 'text-safe',
  failed: 'text-danger'
}

const { follow } = useExternalLink()
const delivery = computed(() => deliveryOf(props.task))
</script>

<template>
  <span class="flex min-w-0 items-center gap-2 truncate font-mono text-caption" data-testid="task-row-delivery" @click.stop @keydown.enter.stop>
    <component
      :is="delivery.issue.url ? 'a' : 'span'"
      v-if="delivery.issue"
      :href="delivery.issue.url ?? undefined"
      target="_blank"
      rel="noopener noreferrer"
      class="focus-ring rounded-key text-text-muted"
      :class="delivery.issue.url && 'underline-offset-4 hover:text-text hover:underline'"
      @click="delivery.issue.url && follow($event, delivery.issue.url)"
    >
      {{ delivery.issue.label }}
    </component>
    <component
      :is="delivery.request.url ? 'a' : 'span'"
      v-if="delivery.request"
      :href="delivery.request.url ?? undefined"
      target="_blank"
      rel="noopener noreferrer"
      class="focus-ring truncate rounded-key"
      :class="[REQUEST_TONE[delivery.request.state], delivery.request.url && 'underline-offset-4 hover:underline']"
      :title="delivery.request.failure ?? undefined"
      :data-state="delivery.request.state"
      @click="delivery.request.url && follow($event, delivery.request.url)"
    >
      {{ delivery.request.label }} {{ delivery.request.state }}
    </component>
  </span>
</template>
