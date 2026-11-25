<script setup lang="ts">
import { computed } from 'vue'
import ProjectCompositionBar from '@/features/catalog/components/ProjectCompositionBar.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import TokenCount from '@/components/ui/TokenCount.vue'
import { relativeTime } from '@/common/format/relative-time'
import { identityHue } from '@/common/identity'
import { PROJECT_STATUS_LABEL, PROJECT_STATUS_TONE } from '@/features/catalog/model/project-status'
import type { ProjectComposition } from '@/features/catalog/lib/project-composition'
import type { Project } from '@/features/catalog/model/catalog'
import type { TokenTotals } from '@/features/tokens/model/token-usage'

/**
 * One project in the ledger: who it is, where its work stands, what it cost and when it last moved.
 * The whole row opens the project; only the delete key sits above that.
 */
const props = withDefaults(
  defineProps<{
    project: Project
    composition: ProjectComposition
    reviewing: number
    tokens: TokenTotals
    deletable?: boolean
  }>(),
  { deletable: true }
)

const emit = defineEmits<{ open: []; delete: [] }>()

const hueLine = computed(() => `linear-gradient(90deg, transparent, ${identityHue(props.project.id).base}, transparent)`)
const taskLabel = computed(() => {
  const count = props.project.taskCount
  return count === 0 ? 'No tasks' : `${count} task${count === 1 ? '' : 's'}`
})
</script>

<template>
  <div
    class="group/row project-row-columns relative min-h-16 border-t border-border py-2.5 transition-colors duration-(--motion-quick) focus-within:bg-surface-raised hover:bg-surface-raised"
  >
    <span
      class="pointer-events-none absolute inset-x-4 -top-px h-px opacity-50 transition-opacity group-hover/row:opacity-100"
      :style="{ background: hueLine }"
      aria-hidden="true"
    />
    <button
      type="button"
      class="focus-ring absolute inset-0"
      :aria-label="`Open ${project.title}`"
      data-testid="project-card"
      @click="emit('open')"
    />

    <div class="pointer-events-none flex min-w-0 items-center gap-3">
      <ProjectMark :project-id="project.id" :icon="project.icon" :image-version="project.iconImageVersion" :size="36" />
      <div class="grid min-w-0 justify-items-start gap-1">
        <p class="max-w-full truncate text-body font-semibold leading-tight text-text">{{ project.title }}</p>
        <span class="anchor-chip px-1.5 text-micro">{{ project.anchor }}</span>
      </div>
    </div>

    <div class="pointer-events-none grid min-w-0 gap-1.5 @max-[52rem]:col-span-2 @max-[52rem]:row-start-2">
      <ProjectCompositionBar :composition="composition" />
      <p class="flex flex-wrap items-center gap-x-3 gap-y-1 text-caption tabular-nums text-text-subtle">
        <span>{{ taskLabel }}</span>
        <span v-if="reviewing > 0" class="font-medium text-accent" data-testid="project-needs">{{ reviewing }} to review</span>
        <span v-if="composition.active > 0" class="inline-flex items-center gap-1.5 text-session-strong">
          <span class="size-1.5 animate-pulse rounded-full bg-session" aria-hidden="true" />
          {{ composition.active }} in progress
        </span>
      </p>
    </div>

    <TokenCount
      class="pointer-events-none justify-self-end @max-[52rem]:row-start-3 @max-[52rem]:justify-self-start"
      :totals="tokens"
      :hue-of="project.id"
    />

    <StatusPill
      class="pointer-events-none justify-self-end @max-[52rem]:col-start-2 @max-[52rem]:row-start-1"
      :tone="PROJECT_STATUS_TONE[project.status]"
    >
      {{ PROJECT_STATUS_LABEL[project.status] }}
    </StatusPill>

    <span
      class="pointer-events-none justify-self-end whitespace-nowrap text-caption text-text-subtle @max-[52rem]:col-start-2 @max-[52rem]:row-start-3"
    >
      {{ relativeTime(project.updatedAt) }}
    </span>

    <button
      v-if="deletable"
      type="button"
      class="focus-ring relative z-10 grid size-7 place-items-center rounded-key text-text-subtle opacity-0 transition hover:bg-surface-hover hover:text-danger focus-visible:opacity-100 group-hover/row:opacity-100 @max-[52rem]:hidden"
      :aria-label="`Delete ${project.title}`"
      data-testid="delete-project"
      @click.stop="emit('delete')"
    >
      <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M5 7h14M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2m1 0-.6 12.4a2 2 0 0 1-2 1.6H9.6a2 2 0 0 1-2-1.6L7 7"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  </div>
</template>
