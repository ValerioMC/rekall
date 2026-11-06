<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import ConnectionProjectBinder from '@/features/connection/components/ConnectionProjectBinder.vue'
import ConnectionUnbindButton from '@/features/connection/components/ConnectionUnbindButton.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { bindableProjects, bindingsOfCompany } from '@/features/connection/lib/connection-impact'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useConsoleStore } from '@/stores/console.store'
import type { ConnectionDetail, ConnectionProject } from '@/features/connection/model/connection'
import type { CompanyId } from '@/model/branded'

/**
 * Who a connection is shared with, as one tree: each company, and under it the projects that bind
 * the connection with their target. A company's project is bound here and unbound here; a company
 * cannot stop sharing while one still binds it, which the companies dialog says.
 */
const props = defineProps<{ detail: ConnectionDetail | null }>()
const emit = defineEmits<{ editCompanies: [] }>()

const store = useConnectionsStore()
const consoleStore = useConsoleStore()
const { run } = useAsyncAction()
const companyBinding = ref<CompanyId | null>(null)

const groups = computed(() =>
  (props.detail?.companies ?? []).map((company) => ({
    company,
    bindings: bindingsOfCompany(props.detail!, company.id),
    projects: bindableProjects(consoleStore.projects, company.id)
  }))
)

async function unbind(binding: ConnectionProject): Promise<void> {
  const detail = props.detail
  if (!detail) return
  await run(() => store.unbindProject(detail.id, binding), `${binding.title} no longer uses ${detail.title}`)
}
</script>

<template>
  <section aria-labelledby="connection-sharing-heading" class="flex flex-col gap-3" data-testid="sharing-panel">
    <header class="flex items-start gap-2">
      <div class="flex-1">
        <h3 id="connection-sharing-heading" class="text-body font-semibold text-text">Shared with</h3>
        <p class="mt-0.5 text-small text-text-subtle">Share it with a company, then choose the projects of that company that use it.</p>
      </div>
      <AppButton size="sm" variant="secondary" class="max-sm:min-h-11" :disabled="!detail" data-testid="edit-companies" @click="emit('editCompanies')"
        >Edit companies</AppButton
      >
    </header>

    <div v-if="!detail" class="skeleton h-16 rounded-control" aria-hidden="true" />
    <p v-else-if="!groups.length" class="text-small text-text-subtle" data-testid="sharing-none">
      Not shared with any company. No project can bind it until a company is added.
    </p>
    <ul v-else class="flex flex-col gap-3">
      <li
        v-for="group in groups"
        :key="group.company.id"
        class="rounded-control border border-border bg-canvas/60 px-3 py-2.5"
        data-testid="sharing-company"
      >
        <div class="flex items-center gap-2">
          <p class="flex-1 text-body font-medium text-text">{{ group.company.name }}</p>
          <AppButton
            v-if="companyBinding !== group.company.id"
            size="sm"
            variant="secondary"
            class="max-sm:min-h-11"
            :disabled="!group.projects.length"
            :title="group.projects.length ? undefined : `${group.company.name} has no active project`"
            :aria-label="`Use ${detail?.title} in a project of ${group.company.name}`"
            data-testid="bind-project"
            @click="companyBinding = group.company.id"
            >+ Project</AppButton
          >
        </div>
        <p v-if="!group.bindings.length" class="text-caption text-text-subtle" data-testid="sharing-unbound">
          {{ group.projects.length ? 'No project uses it yet. Use + Project to choose one.' : 'This company has no active project to use it.' }}
        </p>
        <ul v-else class="mt-1 flex flex-col divide-y divide-border">
          <li
            v-for="binding in group.bindings"
            :key="`${binding.projectId}-${binding.bindingLabel}`"
            class="grid grid-cols-[minmax(0,9rem)_minmax(0,7rem)_minmax(0,1fr)_auto] items-center gap-x-3 gap-y-1 py-1.5 max-sm:grid-cols-[minmax(0,1fr)_auto]"
            data-testid="sharing-binding"
          >
            <RouterLink
              :to="`/projects/${binding.projectId}`"
              class="focus-ring rounded-key text-small text-text hover:text-accent"
              >{{ binding.title }}</RouterLink
            >
            <span class="font-mono text-caption text-anchor/80">{{ binding.bindingLabel }}</span>
            <span class="truncate font-mono text-caption text-text-muted max-sm:col-span-2">{{ binding.target ?? '' }}</span>
            <ConnectionUnbindButton
              class="max-sm:col-span-2 max-sm:justify-self-end"
              :subject="binding.title"
              testid="unbind-binding"
              @confirm="unbind(binding)"
            />
          </li>
        </ul>
        <ConnectionProjectBinder
          v-if="detail && companyBinding === group.company.id"
          :connection-id="detail.id"
          :connection-title="detail.title"
          :kind="detail.kind"
          :projects="group.projects"
          @close="companyBinding = null"
          @bound="companyBinding = null"
        />
      </li>
    </ul>
  </section>
</template>
