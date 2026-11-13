<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import WorkRowView from '@/features/work/components/WorkRow.vue'
import { formatTokens } from '@/common/format/token-count'
import type { TaskId } from '@/model/branded'
import type { WorkDensity, WorkGroup } from '@/features/work/model/work-view'

const props = defineProps<{ group: WorkGroup; density: WorkDensity; now: number; filedUnfolded: boolean }>()

defineEmits<{ open: [taskId: TaskId]; toggleFiled: [] }>()

const taskCountLabel = computed(() => `${props.group.rows.length} ${props.group.rows.length === 1 ? 'task' : 'tasks'}`)
</script>

<template>
  <section :aria-label="group.projectTitle" data-testid="work-group">
    <header class="flex h-[54px] items-center gap-2.5 px-3 pt-4">
      <ProjectMark :project-id="group.projectId" :icon="group.icon" :image-version="group.iconImageVersion" :size="20" />
      <RouterLink
        :to="`/projects/${group.projectId}`"
        class="focus-ring text-body font-semibold text-text hover:text-accent-strong"
      >
        {{ group.projectTitle }}
      </RouterLink>
      <span class="text-small text-text-muted">{{ group.companyName }}</span>
      <span class="font-mono text-caption text-text-subtle">
        {{ taskCountLabel }}
        <template v-if="group.needs > 0"> · <span class="text-accent-strong">{{ group.needs }} to review</span></template>
      </span>
      <span v-if="group.tokens > 0" class="ml-auto font-mono text-caption text-text-subtle">
        {{ formatTokens(group.tokens) }} tokens
      </span>
    </header>

    <WorkRowView
      v-for="row in group.rows"
      :key="row.task.id"
      :row="row"
      :density="density"
      :now="now"
      @open="$emit('open', row.task.id)"
    />

    <template v-if="group.filed.length > 0">
      <button
        type="button"
        class="focus-ring flex h-[34px] w-full cursor-pointer items-center gap-2 rounded-control px-3 text-small text-filed transition-colors duration-(--motion-quick) hover:bg-filed-soft"
        :aria-expanded="filedUnfolded"
        data-testid="work-filed-fold"
        @click="$emit('toggleFiled')"
      >
        <svg
          class="size-3 transition-transform duration-(--motion-move)"
          :class="filedUnfolded && 'rotate-90'"
          viewBox="0 0 12 12"
          fill="none"
          aria-hidden="true"
        >
          <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        {{ group.filed.length }} filed
      </button>
      <template v-if="filedUnfolded">
        <WorkRowView
          v-for="row in group.filed"
          :key="row.task.id"
          :row="row"
          :density="density"
          :now="now"
          @open="$emit('open', row.task.id)"
        />
      </template>
    </template>
  </section>
</template>
