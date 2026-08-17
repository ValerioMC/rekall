<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import CatalogNav from '@/components/catalog/CatalogNav.vue'
import RecordDialog from '@/components/console/RecordDialog.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { useConsoleStore } from '@/stores/console.store'
import { companyDraft } from '@/model/record-draft'
import type { RecordDraft } from '@/model/record-draft'
import type { Company } from '@/model/catalog'

/**
 * Companies, as a page rather than the top level of the `ScopePicker` tree. Editing stays
 * exactly where it already was — `RecordDialog` — because nothing about company editing was
 * the thing asked for here; only somewhere better than a popover to see them from was.
 */
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
    <AppPageHeader title="Companies">
      <template #back>
        <CatalogNav />
      </template>
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
    </AppPageHeader>

    <div class="mx-auto max-w-[1240px] px-8 py-6">
      <div class="mb-5 w-full max-w-[320px]">
        <AppInput v-model="search" type="search" placeholder="Find a company" />
      </div>

      <div v-if="store.isLoading" class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3" aria-hidden="true">
        <div v-for="card in 6" :key="card" class="skeleton h-[120px] rounded-[var(--radius-card)]" />
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

      <div v-else class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <AppCard
          v-for="company in visible"
          :key="company.id"
          interactive
          data-testid="company-card"
          class="group/card relative cursor-pointer"
          @click="openProjectsOf(company)"
        >
          <h3 class="truncate text-[14.5px] font-semibold text-text">{{ company.name }}</h3>
          <p v-if="company.description" class="mt-1.5 line-clamp-2 text-[12.5px] leading-relaxed text-text-muted">
            {{ company.description }}
          </p>
          <p v-else class="mt-1.5 text-[12.5px] italic text-text-subtle">No description yet.</p>
          <p class="mt-3 flex flex-wrap items-center gap-2 text-[11.5px] text-text-subtle">
            <span>{{ company.projectCount }} project{{ company.projectCount === 1 ? '' : 's' }}</span>
            <span aria-hidden="true">&middot;</span>
            <span>{{ company.taskCount }} task{{ company.taskCount === 1 ? '' : 's' }}</span>
            <span aria-hidden="true">&middot;</span>
            <span>{{ relativeTime(company.updatedAt) }}</span>
          </p>

          <div
            class="absolute right-3 top-3 flex gap-1 opacity-0 transition focus-within:opacity-100 group-hover/card:opacity-100"
          >
            <button
              class="focus-ring grid size-6 place-items-center rounded text-text-subtle hover:bg-surface-hover hover:text-accent"
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
              class="focus-ring grid size-6 place-items-center rounded text-text-subtle hover:bg-surface-hover hover:text-danger"
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
          </div>
        </AppCard>
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
