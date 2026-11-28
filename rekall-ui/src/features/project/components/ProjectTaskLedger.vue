<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import ProjectTaskHead from '@/features/project/components/ProjectTaskHead.vue'
import ProjectTaskPager from '@/features/project/components/ProjectTaskPager.vue'
import ProjectTaskPreview from '@/features/project/components/ProjectTaskPreview.vue'
import ProjectTaskRow from '@/features/project/components/ProjectTaskRow.vue'
import { useElementWidth } from '@/composables/useElementWidth'
import { useNow } from '@/composables/useNow'
import { gridTemplate } from '@/features/project/lib/task-columns'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'

const MINUTE_MS = 60_000

const projectTasks = useProjectTasksStore()
const { view, columns, paging, previewsBeside, previewed, foldedEpics, isNarrowed } = storeToRefs(projectTasks)
const ledger = ref<HTMLElement | null>(null)
const width = useElementWidth(ledger)
const now = useNow(MINUTE_MS)

const template = computed(() => gridTemplate(columns.value))
const isEmpty = computed(() => paging.value.total === 0)
const nothingWaits = computed(() => view.value === 'needs' && !isNarrowed.value)

async function showFirstRows(): Promise<void> {
  await nextTick()
  ledger.value?.scrollIntoView?.({ block: 'start' })
}

watch(width, projectTasks.measure)
watch(now, projectTasks.tick, { immediate: true })
watch(() => paging.value.page, showFirstRows)
</script>

<template>
  <div ref="ledger" class="flex flex-1 gap-4" data-testid="task-ledger">
    <div class="flex min-w-0 flex-1 flex-col" :style="{ '--ledger-template': template }">
      <ProjectTaskHead />

      <template v-for="entry in paging.items" :key="entry.row.task.id">
        <template v-if="entry.kind === 'epic'">
          <ProjectTaskRow
            :row="entry.row"
            :now="now"
            :epic="{ members: entry.members, folded: foldedEpics.has(entry.row.task.id) }"
            @toggle-epic="projectTasks.toggleEpic(entry.row.task.id)"
          />
          <template v-if="!foldedEpics.has(entry.row.task.id)">
            <ProjectTaskRow v-for="member in entry.members" :key="member.task.id" :row="member" :now="now" nested />
          </template>
        </template>
        <ProjectTaskRow v-else :row="entry.row" :now="now" :epic-title="entry.epicTitle" />
      </template>

      <div v-if="isEmpty && nothingWaits" class="grid justify-items-center gap-3 py-16 text-center" data-testid="task-nothing-waits">
        <span class="size-14"><StepSeal state="DONE" complete arriving /></span>
        <p class="text-heading font-medium text-text">Nothing waits for you.</p>
        <p class="text-body text-text-muted">Every claimed step is accepted. Resume a session to get more.</p>
      </div>
      <div v-else-if="isEmpty && isNarrowed" class="grid justify-items-center gap-3 px-4 py-16 text-center" data-testid="task-no-match">
        <p class="text-title font-semibold text-text">No task matches.</p>
        <p class="text-body text-text-subtle">Nothing in this view passes the search and the filters.</p>
        <AppButton size="sm" data-testid="task-no-match-clear" @click="projectTasks.clearNarrowing">Clear search and filters</AppButton>
      </div>
      <p v-else-if="isEmpty" class="px-4 py-16 text-center text-body text-text-subtle" data-testid="task-view-empty">
        No task in this view.
      </p>

      <ProjectTaskPager v-if="!isEmpty" />
    </div>

    <ProjectTaskPreview v-if="previewsBeside && previewed" :row="previewed" />
  </div>
</template>
