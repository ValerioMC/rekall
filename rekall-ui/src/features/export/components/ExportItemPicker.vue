<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import type { PageGroup } from '@/features/export/lib/export-items'
import type { ExportSelection } from '@/features/export/model/project-export'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectComponentId, ProjectSectionId } from '@/model/branded'

/** The tick lists of the export dialog: the blueprint, the pages in their contexts and the components, each group with All and None. */
const props = defineProps<{
  selection: ExportSelection
  blueprintEmpty: boolean
  pageGroups: readonly PageGroup[]
  components: readonly ProjectComponent[]
  disabled: boolean
}>()

const emit = defineEmits<{
  blueprint: [on: boolean]
  page: [id: ProjectSectionId]
  component: [id: ProjectComponentId]
  pages: [ids: readonly ProjectSectionId[], on: boolean]
  components: [ids: readonly ProjectComponentId[], on: boolean]
}>()

const pageIds = computed(() => props.pageGroups.flatMap((group) => group.pages.map((page) => page.id)))
const componentIds = computed(() => props.components.map((component) => component.id))
</script>

<template>
  <div class="flex flex-col gap-4" :aria-busy="disabled" data-testid="export-picker">
    <fieldset class="m-0 min-w-0 border-0 p-0">
      <legend class="section-label mb-1.5">Blueprint</legend>
      <div class="flex h-8 items-center">
        <AppCheckbox
          :model-value="selection.blueprint"
          :label="blueprintEmpty ? 'Blueprint, empty' : 'Blueprint'"
          :disabled="disabled || blueprintEmpty"
          @update:model-value="emit('blueprint', $event)"
        />
      </div>
    </fieldset>

    <fieldset v-if="pageGroups.length > 0" class="m-0 min-w-0 border-0 p-0" data-testid="export-pages">
      <legend class="mb-1.5 flex w-full items-baseline gap-2">
        <span class="section-label">Pages</span>
        <span class="font-mono text-micro tabular-nums text-text-subtle">{{ selection.sectionIds.length }}/{{ pageIds.length }}</span>
        <span class="ml-auto flex">
          <AppButton variant="quiet" size="sm" :disabled="disabled" data-testid="export-pages-all" @click="emit('pages', pageIds, true)">All</AppButton>
          <AppButton variant="quiet" size="sm" :disabled="disabled" data-testid="export-pages-none" @click="emit('pages', pageIds, false)">None</AppButton>
        </span>
      </legend>
      <div v-for="group in pageGroups" :key="group.context.id" class="mb-1.5 last:mb-0">
        <p class="mb-0.5 text-caption text-text-muted">{{ group.context.title }}</p>
        <div v-for="page in group.pages" :key="page.id" class="flex h-8 items-center pl-3">
          <AppCheckbox
            :model-value="selection.sectionIds.includes(page.id)"
            :label="page.title"
            :disabled="disabled"
            @update:model-value="emit('page', page.id)"
          />
        </div>
      </div>
    </fieldset>

    <fieldset v-if="components.length > 0" class="m-0 min-w-0 border-0 p-0" data-testid="export-components">
      <legend class="mb-1.5 flex w-full items-baseline gap-2">
        <span class="section-label">Components</span>
        <span class="font-mono text-micro tabular-nums text-text-subtle">{{ selection.componentIds.length }}/{{ components.length }}</span>
        <span class="ml-auto flex">
          <AppButton variant="quiet" size="sm" :disabled="disabled" data-testid="export-components-all" @click="emit('components', componentIds, true)">All</AppButton>
          <AppButton variant="quiet" size="sm" :disabled="disabled" data-testid="export-components-none" @click="emit('components', componentIds, false)">None</AppButton>
        </span>
      </legend>
      <div v-for="component in components" :key="component.id" class="flex h-8 items-center gap-3">
        <AppCheckbox
          :model-value="selection.componentIds.includes(component.id)"
          :label="component.name"
          :disabled="disabled"
          @update:model-value="emit('component', component.id)"
        />
        <span class="min-w-0 truncate font-mono text-caption text-text-subtle" :title="component.folder">{{ component.folder }}</span>
      </div>
    </fieldset>
  </div>
</template>
