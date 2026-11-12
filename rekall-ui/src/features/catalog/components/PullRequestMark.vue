<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import { requestKindOf } from '@/features/catalog/lib/pull-request-request'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * The pull request (GitLab: merge request) of the task's branch: a link to it, 'Merged' once it
 * is, and a warning with the stored reason when it was not opened or not merged. A task of a
 * project with the flow off holds none of these and shows nothing.
 */
const props = defineProps<{
  task: Pick<Task, 'pullRequestNumber' | 'pullRequestUrl' | 'pullRequestFailure' | 'pullRequestMergedAt'>
}>()

const kind = computed(() => requestKindOf(props.task.pullRequestUrl))
const merged = computed(() => props.task.pullRequestMergedAt !== null)
const opened = computed(() => props.task.pullRequestNumber !== null)
const notOpened = computed(() => !opened.value && props.task.pullRequestFailure !== null)
const notMerged = computed(() => opened.value && !merged.value && props.task.pullRequestFailure !== null)
const visible = computed(() => opened.value || notOpened.value)
</script>

<template>
  <span v-if="visible" class="inline-flex shrink-0 items-center gap-1">
    <a
      v-if="task.pullRequestUrl"
      :href="task.pullRequestUrl"
      target="_blank"
      rel="noopener noreferrer"
      class="focus-ring rounded-control"
      :aria-label="`Open ${kind.noun} ${kind.prefix}${task.pullRequestNumber} on the host`"
      data-testid="pull-request-link"
    >
      <AppBadge tone="accent" mono>{{ kind.short }} {{ kind.prefix }}{{ task.pullRequestNumber }} &#8599;</AppBadge>
    </a>
    <AppBadge v-if="merged" tone="safe" data-testid="pull-request-merged">Merged</AppBadge>
    <AppBadge v-if="notOpened" tone="warn" :title="task.pullRequestFailure ?? undefined" data-testid="pull-request-not-opened">
      Not opened
    </AppBadge>
    <AppBadge v-if="notMerged" tone="warn" :title="task.pullRequestFailure ?? undefined" data-testid="pull-request-not-merged">
      Not merged
    </AppBadge>
  </span>
</template>
