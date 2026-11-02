<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppSelect, { type SelectOption } from '@/components/ui/AppSelect.vue'
import { CONNECTION_STATUSES, CONNECTION_STATUS_LABEL, activeFilterCount } from '@/features/connection/lib/connection-filter'
import type { ConnectionFilter } from '@/features/connection/lib/connection-filter'
import { KIND_ORDER } from '@/features/connection/lib/connection-order'
import { CONNECTION_KIND_LABEL } from '@/features/connection/model/connection'
import { asCompanyId, type CompanyId } from '@/model/branded'

/**
 * The hub's three filters: a company ("shared with"), a kind and a status. They narrow the list
 * and never reorder it. Below 640px they fold behind one "Filters (n)" button.
 */
const props = defineProps<{
  filter: ConnectionFilter
  companies: readonly { readonly id: CompanyId; readonly name: string }[]
  hideClear?: boolean
}>()
const emit = defineEmits<{ change: [filter: ConnectionFilter]; clear: [] }>()

const open = ref(false)
const count = computed(() => activeFilterCount(props.filter))
const companyOptions = computed<SelectOption[]>(() => props.companies.map((company) => ({ value: company.id, label: company.name })))
const kindOptions: SelectOption[] = KIND_ORDER.map((kind) => ({ value: kind, label: CONNECTION_KIND_LABEL[kind] }))
const statusOptions: SelectOption[] = CONNECTION_STATUSES.map((status) => ({ value: status, label: CONNECTION_STATUS_LABEL[status] }))

function changeCompany(value: string): void {
  emit('change', { ...props.filter, companyId: value ? asCompanyId(value) : null })
}

function changeKind(value: string): void {
  emit('change', { ...props.filter, kind: KIND_ORDER.find((kind) => kind === value) ?? null })
}

function changeStatus(value: string): void {
  emit('change', { ...props.filter, status: CONNECTION_STATUSES.find((status) => status === value) ?? null })
}
</script>

<template>
  <div class="flex flex-col gap-2 px-4 py-3 sm:flex-row sm:flex-wrap sm:items-center" role="search" aria-label="Filter connections">
    <AppButton
      class="sm:hidden max-sm:min-h-11"
      :aria-expanded="open"
      aria-controls="connection-filters"
      data-testid="filters-toggle"
      @click="open = !open"
    >
      Filters ({{ count }})
    </AppButton>
    <div id="connection-filters" class="flex-col gap-2 sm:flex sm:flex-row sm:flex-wrap sm:items-center" :class="open ? 'flex' : 'hidden'">
      <AppSelect
        class="sm:w-44"
        aria-label="Shared with company"
        :model-value="filter.companyId"
        :options="companyOptions"
        placeholder="Any company"
        data-testid="filter-company"
        @update:model-value="changeCompany"
      />
      <AppSelect
        class="sm:w-36"
        aria-label="Kind"
        :model-value="filter.kind"
        :options="kindOptions"
        placeholder="Any kind"
        data-testid="filter-kind"
        @update:model-value="changeKind"
      />
      <AppSelect
        class="sm:w-52"
        aria-label="Status"
        :model-value="filter.status"
        :options="statusOptions"
        placeholder="Any status"
        data-testid="filter-status"
        @update:model-value="changeStatus"
      />
      <button
        v-if="count && !hideClear"
        type="button"
        class="focus-ring rounded-key px-1 text-small text-accent underline-offset-2 hover:underline max-sm:min-h-11 max-sm:text-left"
        data-testid="clear-filters"
        @click="emit('clear')"
      >
        Clear filters
      </button>
    </div>
  </div>
</template>
