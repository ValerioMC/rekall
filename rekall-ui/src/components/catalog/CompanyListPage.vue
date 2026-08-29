<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
import RecordDialog from '@/components/console/RecordDialog.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppInput from '@/components/ui/AppInput.vue'
import StatusMixBar from '@/components/ui/StatusMixBar.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { identityHue } from '@/common/identity'
import { useConsoleStore } from '@/stores/console.store'
import { companyDraft } from '@/model/record-draft'
import type { RecordDraft } from '@/model/record-draft'
import type { Company, Task } from '@/model/catalog'
import type { CompanyId } from '@/model/branded'

const store = useConsoleStore()
const router = useRouter()
const { run } = useAsyncAction()

const search = ref('')
const editing = ref<RecordDraft | null>(null)
const deleting = ref<Company | null>(null)

const visible = computed(() => {
  const needle = search.value.toLowerCase().trim()
  if (!needle) return store.companies
  return store.companies.filter((company) => company.name.toLowerCase().includes(needle))
})

function openProjectsOf(company: Company): void {
  router.push({ path: '/projects', query: { company: company.id } })
}

function edit(company: Company): void {
  editing.value = companyDraft(company)
}

function tasksOfCompany(companyId: CompanyId): Task[] {
  const projectIds = new Set(store.projects.filter((p) => p.companyId === companyId).map((p) => p.id))
  return store.tasks.filter((t) => projectIds.has(t.projectId))
}

function blastOf(company: Company): string {
  return `${company.projectCount} project${company.projectCount === 1 ? '' : 's'} · ${company.taskCount} task${company.taskCount === 1 ? '' : 's'} · every note left on nothing`
}

async function confirmDelete(): Promise<void> {
  const company = deleting.value
  if (!company) return
  await run(() => store.deleteCompany(company.id), `Deleted ${company.name}`)
  deleting.value = null
}
</script>

<template>
  <div class="min-h-full bg-canvas">
    <AppCatalogHeader title="Companies">
      <template #actions>
        <AppButton
          variant="primary"
          size="sm"
          data-testid="new-company"
          @click="editing = companyDraft()"
        >
          + New company
        </AppButton>
      </template>
    </AppCatalogHeader>

    <div class="mx-auto max-w-[1240px] px-8 py-6">
      <div class="mb-5 w-full max-w-[320px]">
        <AppInput v-model="search" type="search" placeholder="Find a company" />
      </div>

      <div v-if="store.isLoading" class="flex flex-col gap-2" aria-hidden="true">
        <div v-for="row in 5" :key="row" class="skeleton h-[64px] rounded-[var(--radius-control)]" />
      </div>

      <AppEmptyState
        v-else-if="!visible.length && !store.companies.length"
        title="No company yet"
        description="A company is the outermost record: it holds the projects, and those hold the work."
      >
        <AppButton variant="secondary" size="sm" @click="editing = companyDraft()">
          + New company
        </AppButton>
      </AppEmptyState>

      <AppEmptyState v-else-if="!visible.length" title="No company matches" description="Try a different search." />

      <div v-else class="overflow-hidden rounded-[var(--radius-card)] border border-border bg-surface">
        <button
          v-for="(company, index) in visible"
          :key="company.id"
          data-testid="company-card"
          class="focus-ring group/row relative flex w-full items-center gap-4 py-3 pl-4 pr-4 text-left transition-colors hover:bg-surface-raised"
          :class="index > 0 ? 'border-t border-border' : ''"
          @click="openProjectsOf(company)"
        >
          <span
            class="grid size-9 shrink-0 place-items-center rounded-[9px] border font-mono text-[13px] font-semibold"
            :style="{
              backgroundColor: identityHue(company.id).soft,
              borderColor: identityHue(company.id).line,
              color: identityHue(company.id).base
            }"
            aria-hidden="true"
          >
            {{ company.name.charAt(0).toUpperCase() }}
          </span>

          <span class="min-w-0 flex-1">
            <span class="block truncate text-[14px] font-semibold text-text">{{ company.name }}</span>
            <span v-if="company.description" class="mt-0.5 block truncate text-[12px] text-text-subtle">
              {{ company.description }}
            </span>
            <StatusMixBar :tasks="tasksOfCompany(company.id)" class="mt-1.5 max-w-[180px]" />
          </span>

          <span class="hidden shrink-0 items-center gap-2 text-[11.5px] text-text-subtle sm:flex">
            <span>{{ company.projectCount }} project{{ company.projectCount === 1 ? '' : 's' }}</span>
            <span aria-hidden="true">&middot;</span>
            <span>{{ company.taskCount }} task{{ company.taskCount === 1 ? '' : 's' }}</span>
            <span aria-hidden="true">&middot;</span>
            <span>{{ relativeTime(company.updatedAt) }}</span>
          </span>

          <span class="flex shrink-0 gap-1 opacity-0 transition focus-within:opacity-100 group-hover/row:opacity-100">
            <button
              class="focus-ring grid size-7 place-items-center rounded text-text-subtle hover:bg-surface-hover hover:text-accent"
              :aria-label="`Edit ${company.name}`"
              data-testid="edit-company"
              @click.stop="edit(company)"
            >
              <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                <path
                  d="M4 20h4L20 8l-4-4L4 16v4z"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linejoin="round"
                />
              </svg>
            </button>
            <button
              class="focus-ring grid size-7 place-items-center rounded text-text-subtle hover:bg-surface-hover hover:text-danger"
              :aria-label="`Delete ${company.name}`"
              data-testid="delete-company"
              @click.stop="deleting = company"
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
          </span>

          <span
            class="shrink-0 text-[13px] text-text-subtle transition-transform group-hover/row:translate-x-0.5 group-hover/row:text-accent"
            aria-hidden="true"
          >
            &#8594;
          </span>
        </button>
      </div>
    </div>

    <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />

    <AppConfirm
      v-if="deleting"
      :title="`Delete ${deleting.name}?`"
      body="Everything underneath goes with it. This is not recoverable."
      :blast="blastOf(deleting)"
      confirm-label="Delete company"
      @cancel="deleting = null"
      @confirm="confirmDelete"
    />
  </div>
</template>
