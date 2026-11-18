<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import { useEpicConversion } from '@/features/catalog/composables/useEpicConversion'
import { codedTitle } from '@/features/catalog/model/task-code'
import { TASK_STATUS_LABEL } from '@/features/catalog/model/task-status'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * Turns a task into an epic after naming what stays and what is deleted. A task with nothing to
 * lose gets a plain confirm; one on the run queue cannot be converted until it is taken off.
 */
const props = defineProps<{ task: Task }>()
const emit = defineEmits<{ close: [] }>()

const { conversion, losses, isConverting, convert } = useEpicConversion(props.task.id, () => emit('close'))

const carriedOver = computed(() => {
  const task = props.task
  const rows = [
    { key: 'identity', what: 'Title, code, status', detail: `${task.title} · ${task.code} · ${TASK_STATUS_LABEL[task.status]}` },
    { key: 'description', what: 'Description', detail: 'Becomes the brief every task of the epic inherits' }
  ]
  if (task.documentCount > 0) rows.push({ key: 'notes', what: 'Notes', detail: `${task.documentCount} note${task.documentCount === 1 ? '' : 's'} stay attached` })
  const kept = [task.tagName, task.trackerIssueKey].filter((value): value is string => Boolean(value))
  if (kept.length > 0) rows.push({ key: 'kept', what: 'Label and issue', detail: kept.join(' · ') })
  return rows
})

const confirmLabel = computed(() => (losses.value.length > 0 ? 'Convert and delete' : 'Convert to epic'))

async function confirm(): Promise<void> {
  const converted = await convert(`${codedTitle(props.task)} is an epic now.`)
  if (converted) emit('close')
}
</script>

<template>
  <AppDialog :label="`Convert ${task.code} to an epic`" panel-class="max-w-[600px]" panel-testid="convert-to-epic-dialog" @dismiss="emit('close')">
    <div class="px-6 pb-2 pt-5">
      <h2 class="flex items-center gap-2 text-title font-semibold text-text">
        <EpicGlyph :size="16" />
        Convert <span class="anchor-chip px-1.5 py-px text-small">{{ task.code }}</span> to an epic
      </h2>
      <p class="mt-1.5 max-w-[520px] text-body leading-relaxed text-text-muted">
        An epic gathers tasks and shares a brief; it has no plan and is never run.
        <template v-if="losses.length > 0">Everything an epic can hold moves over. The rest is deleted.</template>
      </p>
    </div>

    <div v-if="!conversion" class="px-6 pb-6 pt-3" aria-hidden="true">
      <div class="skeleton h-24 w-full rounded-control" />
    </div>

    <template v-else>
      <section class="px-6 pt-3" aria-label="Carried over">
        <h3 class="eyebrow mb-1.5 text-caption">Carried over</h3>
        <ul class="divide-y divide-border rounded-control border border-border bg-canvas" data-testid="conversion-carried">
          <li v-for="row in carriedOver" :key="row.key" class="flex items-center gap-3 px-3 py-2 text-small">
            <svg class="size-3 shrink-0 text-safe" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M2.5 6.2l2.4 2.4 4.6-5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <span class="w-36 shrink-0 font-medium text-text">{{ row.what }}</span>
            <span class="min-w-0 truncate text-text-muted">{{ row.detail }}</span>
          </li>
        </ul>
      </section>

      <section v-if="losses.length > 0" class="px-6 pt-4" aria-label="Deleted by the conversion">
        <div class="rounded-control border border-warn/35 bg-warn-soft" data-testid="conversion-losses">
          <p class="flex items-center gap-2 px-3 pb-2 pt-2.5 text-small font-medium text-warn">
            <svg class="size-3.5 shrink-0" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path d="M8 2l6.3 11H1.7L8 2z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round" />
              <path d="M8 6.5v3M8 11.4v.1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            Deleted by the conversion
          </p>
          <ul class="divide-y divide-warn/20 border-t border-warn/20">
            <li v-for="loss in losses" :key="loss.key" class="flex items-baseline gap-3 px-3 py-2 text-small" data-testid="conversion-loss">
              <span class="w-36 shrink-0 font-medium text-text">{{ loss.what }}</span>
              <span class="min-w-0 flex-1 text-text-muted">{{ loss.detail }}</span>
              <span class="min-w-5 text-right font-mono text-caption font-semibold tabular-nums text-warn">{{ loss.count }}</span>
            </li>
          </ul>
        </div>
      </section>
      <p v-else class="px-6 pt-3 text-small text-text-muted">Nothing is deleted: the task has no steps, wrapup, commits or dependencies.</p>

      <p v-if="conversion.queued" class="px-6 pt-3 text-small text-danger" role="alert" data-testid="conversion-queued">
        This task is on the run queue. Take it off the queue before turning it into an epic.
      </p>
    </template>

    <div class="mt-5 flex items-center gap-2 border-t border-border bg-canvas px-6 py-3">
      <span v-if="losses.length > 0" class="mr-auto text-caption text-text-subtle">It cannot be undone.</span>
      <AppButton class="ml-auto" variant="ghost" size="sm" @click="emit('close')">Cancel</AppButton>
      <AppButton
        variant="primary"
        size="sm"
        :disabled="!conversion || conversion.queued"
        :loading="isConverting"
        data-testid="convert-to-epic-confirm"
        @click="confirm"
      >
        {{ confirmLabel }}
      </AppButton>
    </div>
  </AppDialog>
</template>
