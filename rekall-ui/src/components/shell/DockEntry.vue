<script setup lang="ts">
import { computed } from 'vue'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectId } from '@/model/branded'

/**
 * One piece of live work in a dock sheet, led by its company and project. The jump button covers
 * the whole entry, so a control in the slot must be `relative` to stay pressable.
 */
const props = defineProps<{ projectId: ProjectId | null; title: string; jumpTestId: string }>()

const emit = defineEmits<{ jump: [] }>()

const store = useConsoleStore()

const project = computed(() => store.projects.find((candidate) => candidate.id === props.projectId) ?? null)

const logoImageVersion = computed(
  () => store.companies.find((company) => company.id === project.value?.companyId)?.logoImageVersion ?? null
)
</script>

<template>
  <li class="dock-entry">
    <div
      class="dock-entry-row group relative grid grid-cols-[45px_minmax(0,1fr)] gap-x-3 rounded-control p-2.5 transition-colors duration-(--motion-quick) hover:bg-surface-raised"
    >
      <span class="relative mt-0.5 size-9" data-testid="dock-entry-marks">
        <template v-if="project">
          <CompanyMark
            :company-id="project.companyId"
            :name="project.companyName"
            :image-version="logoImageVersion"
            :size="36"
          />
          <span
            class="absolute -bottom-2.25 -right-2.25 grid rounded-control bg-surface ring-2 ring-surface transition-colors duration-(--motion-quick) group-hover:bg-surface-raised group-hover:ring-surface-raised"
          >
            <ProjectMark
              :project-id="project.id"
              :icon="project.icon"
              :image-version="project.iconImageVersion"
              :size="22"
            />
          </span>
        </template>
      </span>
      <div class="min-w-0">
        <button
          type="button"
          class="focus-ring block w-full rounded-control text-left after:absolute after:inset-0"
          :title="`Jump to ${title}`"
          :data-testid="jumpTestId"
          @click="emit('jump')"
        >
          <span
            v-if="project"
            class="mb-0.5 flex items-center gap-1.5 text-caption text-text-muted"
            data-testid="dock-entry-identity"
          >
            <span class="max-w-1/2 shrink-0 truncate">{{ project.companyName }}</span>
            <span class="shrink-0 text-border-strong" aria-hidden="true">/</span>
            <span class="truncate font-medium text-text">{{ project.title }}</span>
          </span>
          <span class="line-clamp-2 text-body font-medium leading-snug text-text">{{ title }}</span>
        </button>
        <slot />
      </div>
    </div>
  </li>
</template>

<style scoped>
/* A hairline parts consecutive entries; it steps aside for the hovered row's own fill. */
.dock-entry + .dock-entry .dock-entry-row::before {
  content: '';
  position: absolute;
  inset: 0 10px auto;
  height: 1px;
  background: var(--color-border);
}

.dock-entry + .dock-entry .dock-entry-row:hover::before {
  background: transparent;
}
</style>
