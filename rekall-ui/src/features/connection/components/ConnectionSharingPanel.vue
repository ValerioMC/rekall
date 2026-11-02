<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { bindingsOfCompany } from '@/features/connection/lib/connection-impact'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { ConnectionDetail, ConnectionProject } from '@/features/connection/model/connection'

/**
 * Who a connection is shared with, as one tree: each company, and under it the projects that bind
 * the connection with their target. A project can be unbound here; a company cannot stop sharing
 * while one still binds it, which the companies dialog says.
 */
const props = defineProps<{ detail: ConnectionDetail | null }>()
const emit = defineEmits<{ editCompanies: [] }>()

const store = useConnectionsStore()
const { run } = useAsyncAction()

const groups = computed(() =>
  (props.detail?.companies ?? []).map((company) => ({ company, bindings: bindingsOfCompany(props.detail!, company.id) }))
)

async function unbind(binding: ConnectionProject): Promise<void> {
  const detail = props.detail
  if (!detail) return
  await run(() => store.unbindProject(detail.id, binding), `${binding.title} no longer uses ${detail.title}`)
}
</script>

<template>
  <section aria-labelledby="connection-sharing-heading" class="flex flex-col gap-3" data-testid="sharing-panel">
    <header class="flex items-center gap-2">
      <h3 id="connection-sharing-heading" class="flex-1 text-small font-semibold text-text">Shared with</h3>
      <AppButton size="sm" variant="secondary" :disabled="!detail" data-testid="edit-companies" @click="emit('editCompanies')"
        >Edit companies</AppButton
      >
    </header>

    <div v-if="!detail" class="skeleton h-16 rounded-control" aria-hidden="true" />
    <p v-else-if="!groups.length" class="text-small text-text-subtle" data-testid="sharing-none">
      Not shared with any company. No project can bind it until a company is added.
    </p>
    <ul v-else class="flex flex-col gap-3">
      <li v-for="group in groups" :key="group.company.id" data-testid="sharing-company">
        <p class="text-body font-medium text-text">{{ group.company.name }}</p>
        <p v-if="!group.bindings.length" class="text-caption text-text-subtle">Not bound yet</p>
        <ul v-else class="mt-1 flex flex-col divide-y divide-border">
          <li
            v-for="binding in group.bindings"
            :key="`${binding.projectId}-${binding.bindingLabel}`"
            class="flex flex-wrap items-baseline gap-x-3 gap-y-1 py-1.5"
            data-testid="sharing-binding"
          >
            <RouterLink
              :to="`/projects/${binding.projectId}`"
              class="focus-ring rounded-key text-small text-text hover:text-accent"
              >{{ binding.title }}</RouterLink
            >
            <span class="font-mono text-caption text-anchor/80">{{ binding.bindingLabel }}</span>
            <span v-if="binding.target" class="font-mono text-caption text-text-muted">{{ binding.target }}</span>
            <AppButton
              size="sm"
              variant="danger-quiet"
              class="ml-auto max-sm:min-h-11"
              :aria-label="`Unbind ${binding.title} from ${detail.title}`"
              data-testid="unbind-binding"
              @click="unbind(binding)"
              >Unbind</AppButton
            >
          </li>
        </ul>
      </li>
    </ul>
  </section>
</template>
