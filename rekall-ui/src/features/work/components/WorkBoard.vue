<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import WorkBoardLane from '@/features/work/components/WorkBoardLane.vue'
import WorkTaskSheet from '@/features/work/components/WorkTaskSheet.vue'
import { useNow } from '@/composables/useNow'
import { useWorkStore } from '@/features/work/store/work.store'

const MINUTE_MS = 60_000

const work = useWorkStore()
const router = useRouter()
const { lanes, rows, sheetTaskId } = storeToRefs(work)
const now = useNow(MINUTE_MS)

const liveLanes = computed(() => lanes.value.filter((lane) => lane.id !== 'filed'))
const filedCount = computed(() => lanes.value.find((lane) => lane.id === 'filed')?.rows.length ?? 0)
const sheetRow = computed(() => rows.value.find((row) => row.task.id === sheetTaskId.value) ?? null)

function openFiledShelf(): void {
  work.view = 'filed'
  void router.push('/')
}
</script>

<template>
  <div
    class="grid min-h-0 flex-1 grid-cols-[repeat(3,minmax(0,1fr))_160px] gap-4 px-8 pb-5.5 pt-1.5 max-[1180px]:grid-cols-[repeat(3,minmax(0,1fr))]"
    data-testid="work-board"
  >
    <WorkBoardLane v-for="lane in liveLanes" :key="lane.id" :lane="lane" :now="now" @open="work.openSheet" />

    <section
      class="flex min-h-0 flex-col rounded-card border border-dashed border-filed-line bg-canvas max-[1180px]:hidden"
      aria-label="Filed"
      data-testid="board-lane-filed"
    >
      <header class="flex h-11 items-center pl-3.5 pr-3 font-medium text-filed">Filed</header>
      <div class="px-3.5">
        <p class="text-display font-light tabular-nums text-filed" data-testid="board-filed-count">{{ filedCount }}</p>
        <p class="mt-1.5 text-small leading-normal text-text-muted">Finished tasks stay out of the way.</p>
        <AppButton class="mt-2 px-0" variant="ghost" size="sm" data-testid="board-open-shelf" @click="openFiledShelf">
          Open the shelf
        </AppButton>
      </div>
    </section>

    <WorkTaskSheet v-if="sheetRow" :row="sheetRow" @close="work.closeSheet()" />
  </div>
</template>
