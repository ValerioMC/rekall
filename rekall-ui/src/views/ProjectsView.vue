<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import { createProject } from '@/api/catalog.api'
import { useCatalogStore } from '@/stores/catalog.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { PROJECT_STATUSES } from '@/model/catalog'

const catalog = useCatalogStore()
const { projects, isLoading } = storeToRefs(catalog)
const { run, isRunning } = useAsyncAction()

const STATUS_OPTIONS = PROJECT_STATUSES.map((status) => ({ value: status, label: status }))

const isCreating = ref(false)
const draft = ref({ name: '', status: 'ACTIVE' as const, description: '' })

onMounted(() => catalog.load())

async function create(): Promise<void> {
  const created = await run(
    () =>
      createProject({
        name: draft.value.name,
        status: draft.value.status,
        description: draft.value.description || null
      }),
    'Project created.'
  )
  if (!created) return
  isCreating.value = false
  draft.value = { name: '', status: 'ACTIVE', description: '' }
  await catalog.load()
}
</script>

<template>
  <AppPageHeader title="Projects">
    <template #actions>
      <AppButton variant="primary" @click="isCreating = !isCreating">
        {{ isCreating ? 'Cancel' : 'New project' }}
      </AppButton>
    </template>
  </AppPageHeader>

  <div class="mx-auto w-full max-w-[1240px] space-y-5 px-8 pb-20 pt-6">
    <AppCard v-if="isCreating">
      <div class="grid gap-4 sm:grid-cols-2">
        <AppField label="Name" hint="What you type after project: in an anchor">
          <AppInput v-model="draft.name" placeholder="STVV" />
        </AppField>
        <AppField label="Status">
          <AppSelect v-model="draft.status" :options="STATUS_OPTIONS" />
        </AppField>
      </div>
      <AppField label="Description" class="mt-4">
        <AppTextarea v-model="draft.description" :rows="3" />
      </AppField>
      <div class="mt-4 flex justify-end">
        <AppButton variant="primary" :loading="isRunning" :disabled="!draft.name" @click="create">
          Create
        </AppButton>
      </div>
    </AppCard>

    <AppSkeleton v-if="isLoading" :lines="4" />

    <AppEmptyState
      v-else-if="!projects.length"
      title="No projects yet"
      description="A project is the outermost anchor. Everything else hangs off one."
    />

    <div v-else class="grid gap-3">
      <RouterLink v-for="project in projects" :key="project.id" :to="`/projects/${project.id}`">
        <AppCard class="transition-colors hover:border-border-strong">
          <div class="flex items-center gap-3">
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span class="text-[15px] font-semibold text-text">{{ project.name }}</span>
                <AppBadge>{{ project.status }}</AppBadge>
              </div>
              <p v-if="project.description" class="mt-1 truncate text-[13px] text-text-muted">
                {{ project.description }}
              </p>
              <code class="mt-1 block font-mono text-[11px] text-text-subtle">
                project:{{ project.name }}
              </code>
            </div>
            <span class="shrink-0 font-mono text-[12px] text-text-subtle">
              {{ project.taskCount }} task{{ project.taskCount === 1 ? '' : 's' }}
            </span>
          </div>
        </AppCard>
      </RouterLink>
    </div>
  </div>
</template>
