<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import ConnectionCompanyPicker from '@/features/connection/components/ConnectionCompanyPicker.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { bindingsOfCompany } from '@/features/connection/lib/connection-impact'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { ConnectionDetail, ConnectionProject } from '@/features/connection/model/connection'
import type { CompanyId } from '@/model/branded'

/** Changes which companies a connection is shared with. A company whose projects still bind it stays checked and locked. */
const props = defineProps<{ detail: ConnectionDetail; companies: readonly { readonly id: CompanyId; readonly name: string }[] }>()
const emit = defineEmits<{ close: [] }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()
const selected = ref<readonly CompanyId[]>(props.detail.companyIds)

const changed = computed(
  () =>
    selected.value.length !== props.detail.companyIds.length ||
    selected.value.some((companyId) => !props.detail.companyIds.includes(companyId))
)

const blockedBy = computed(() => {
  const blocked = new Map<CompanyId, readonly ConnectionProject[]>()
  for (const companyId of props.detail.companyIds) {
    const bindings = bindingsOfCompany(props.detail, companyId)
    if (bindings.length) blocked.set(companyId, bindings)
  }
  return blocked
})

async function save(): Promise<void> {
  const saved = await run(() => store.setCompanies(props.detail.id, selected.value), `Updated who ${props.detail.title} is shared with`)
  if (saved) emit('close')
}
</script>

<template>
  <AppDialog label="Edit companies" panel-class="max-w-[520px]" panel-testid="companies-dialog" @dismiss="emit('close')">
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <span class="min-w-0 flex-1 text-title font-semibold tracking-[-0.01em] text-text">Shared with</span>
      <button
        class="focus-ring grid size-7 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>
    <div class="px-6 py-5">
      <p class="mb-3 text-small text-text-muted">
        Every project of a company that {{ detail.title }} is shared with can bind it.
      </p>
      <ConnectionCompanyPicker v-model="selected" :companies="companies" :blocked-by="blockedBy" />
    </div>
    <footer class="flex items-center justify-end gap-2 border-t border-border bg-canvas px-6 py-3.5">
      <AppButton size="sm" variant="ghost" @click="emit('close')">Cancel</AppButton>
      <AppButton size="sm" variant="primary" :loading="isRunning" :disabled="!changed" data-testid="save-companies" @click="save"
        >Save companies</AppButton
      >
    </footer>
  </AppDialog>
</template>
