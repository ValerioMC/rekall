<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import { reportCloseDate, tokensConsumed } from '@/features/provider-report/lib/provider-report'
import type { ProviderReportPeriod } from '@/features/provider-report/model/provider-report'
import type { TaskId } from '@/model/branded'
import ProviderReportToken from './ProviderReportToken.vue'

const props = defineProps<{ period: ProviderReportPeriod }>()
const router = useRouter()
const sharedProjectNames = computed(() => {
  const companyByProject = new Map<string, string>()
  const shared = new Set<string>()
  for (const { task } of props.period.tasks) {
    const company = companyByProject.get(task.projectTitle)
    if (company !== undefined && company !== task.companyName) shared.add(task.projectTitle)
    companyByProject.set(task.projectTitle, task.companyName)
  }
  return shared
})

async function openRow(event: MouseEvent, id: TaskId): Promise<void> {
  if (event.target instanceof Element && event.target.closest('a, button')) return
  await router.push({ name: 'task', params: { id, tab: 'description' } })
}
</script>

<template>
  <section id="provider-report-task-detail" aria-labelledby="provider-report-tasks">
    <div class="report-section-heading" aria-live="polite">
      <h2 id="provider-report-tasks" class="text-heading font-medium">Tasks · {{ period.label }}</h2>
      <p class="text-small text-text-muted">{{ period.tasks.length }} {{ period.tasks.length === 1 ? 'task' : 'tasks' }}</p>
    </div>
    <div class="report-ledger">
      <table v-if="period.tasks.length" class="report-tasks" aria-labelledby="provider-report-tasks">
        <colgroup><col><col><col><col><col><col><col><col></colgroup>
        <thead>
          <tr>
            <th scope="col">Code</th>
            <th scope="col">Task</th>
            <th scope="col">Project</th>
            <th scope="col">Closed date</th>
            <th scope="col">Provider</th>
            <th scope="col" class="text-right">Steps closed</th>
            <th scope="col" class="text-right">Period tokens</th>
            <th scope="col" class="text-right">All-time tokens</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in period.tasks" :key="row.task.id" class="cursor-pointer hover:bg-surface-raised focus-within:bg-surface-raised" :data-task-code="row.task.code" @click="openRow($event, row.task.id)">
            <td>
              <span class="flex flex-wrap items-center gap-1 whitespace-nowrap font-mono text-caption text-anchor">
                {{ row.task.code }}<CopyAnchorButton :anchor="row.task.anchor" class="!size-5" />
              </span>
            </td>
            <th scope="row" class="font-semibold">
              <RouterLink class="focus-ring text-text underline-offset-2 hover:underline" :to="{ name: 'task', params: { id: row.task.id, tab: 'description' } }">{{ row.task.title }}</RouterLink>
            </th>
            <td class="text-small text-text-muted">
              <span v-if="sharedProjectNames.has(row.task.projectTitle)" class="block text-caption text-text-subtle">{{ row.task.companyName }}</span>
              {{ row.task.projectTitle }}
            </td>
            <td class="whitespace-nowrap text-small text-text-muted">
              <time v-if="row.closedAt" class="text-safe" :datetime="row.closedAt">{{ reportCloseDate(row.closedAt) }}</time>
              <span v-else-if="row.task.status === 'DONE'">Date not recorded</span>
              <span v-else>Still open</span>
            </td>
            <td class="text-small text-text-muted">
              <div v-if="row.providers.length" class="flex flex-col gap-1">
                <span v-for="provider in row.providers" :key="provider" class="inline-flex items-center gap-1.5 whitespace-nowrap"><SessionProviderMark :provider="provider" :size="12" />{{ SESSION_PROVIDER_LABEL[provider] }}</span>
              </div>
              <span v-else>Not recorded</span>
            </td>
            <td class="report-number" :class="{ 'text-text-subtle': row.stepsClosed === 0 }">{{ row.stepsClosed }}</td>
            <td class="report-number">
              <div v-if="row.providers.length" class="flex flex-col gap-1 font-medium">
                <ProviderReportToken v-for="provider in row.providers" :key="provider" :count="tokensConsumed(row.periodTokens[provider])" />
              </div>
              <ProviderReportToken v-else :count="0" />
            </td>
            <td class="report-number text-text-muted">
              <div v-if="row.providers.length" class="flex flex-col gap-1">
                <ProviderReportToken v-for="provider in row.providers" :key="provider" :count="tokensConsumed(row.lifetimeTokens[provider])" />
              </div>
              <ProviderReportToken v-else :count="0" />
            </td>
          </tr>
        </tbody>
      </table>
      <div v-else class="px-4 py-6 text-center">
        <p class="text-title font-medium">{{ period.totalClosed.steps ? 'No tasks to list in this period' : 'No activity in this period' }}</p>
        <p class="mt-1 text-text-muted">Choose another period to see its tasks.</p>
      </div>
    </div>
  </section>
</template>
