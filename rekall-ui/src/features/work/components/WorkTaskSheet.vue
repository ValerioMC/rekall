<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import WorkPeek from '@/features/work/components/WorkPeek.vue'
import { useOpenTask } from '@/features/work/composables/useOpenTask'
import { useConsoleStore } from '@/stores/console.store'
import type { WorkRow } from '@/features/work/model/work-view'

/**
 * A card's task over the board: the same peek a ledger row opens, in a sheet 600px wide. Escape or
 * a press outside closes it; Open as page leaves the board for the task.
 */
const props = defineProps<{ row: WorkRow }>()

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const openTask = useOpenTask()
const sheet = ref<HTMLElement | null>(null)

const project = computed(() => store.projects.find((candidate) => candidate.id === props.row.task.projectId) ?? null)

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') emit('close')
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  sheet.value?.focus()
})
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="fixed inset-x-0 bottom-0 top-(--spacing-header) z-(--z-overlay) bg-canvas/80" data-testid="sheet-scrim" @click="emit('close')" />
  <aside
    ref="sheet"
    class="fixed bottom-0 right-0 top-(--spacing-header) z-(--z-overlay) flex w-[600px] max-w-full flex-col gap-3 overflow-y-auto border-l border-border-strong bg-canvas p-5 shadow-modal outline-none"
    role="dialog"
    :aria-label="row.task.title"
    tabindex="-1"
    data-testid="task-sheet"
  >
    <div class="flex items-center gap-2 text-small text-text-muted">
      <ProjectMark
        :project-id="row.task.projectId"
        :icon="project?.icon ?? 'folder'"
        :image-version="project?.iconImageVersion ?? null"
        :size="16"
      />
      <span class="min-w-0 truncate">{{ row.task.companyName }} / {{ row.task.projectTitle }}</span>
      <span class="flex-1" />
      <AppButton variant="quiet" size="sm" data-testid="sheet-open-page" @click="openTask(row.task.id)">Open as page</AppButton>
      <AppButton variant="ghost" size="icon" aria-label="Close" data-testid="sheet-close" @click="emit('close')">
        <CloseGlyph />
      </AppButton>
    </div>
    <h2 class="text-heading font-medium text-text">{{ row.task.title }}</h2>
    <WorkPeek :row="row" />
  </aside>
</template>
