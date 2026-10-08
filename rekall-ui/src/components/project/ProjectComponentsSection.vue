<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import ProjectComponentCard from '@/components/project/ProjectComponentCard.vue'
import ProjectComponentComposer from '@/components/project/ProjectComponentComposer.vue'
import ProjectComponentTile from '@/components/project/ProjectComponentTile.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { componentsPanelCollapsed, setComponentsPanelCollapsed } from '@/common/config/components-panel'
import { canMove, type Direction } from '@/common/project/section-order'
import { useProjectComponentsStore } from '@/stores/projectComponents.store'
import type { ProjectComponent, ProjectComponentInput } from '@/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

/**
 * The folders of a project that are parts of it. They ride along in every context that loads
 * the project, so a session told "the billing module" knows which folder that is and what it does.
 */
const props = defineProps<{ projectId: ProjectId; projectTitle: string; repoFolder: string | null }>()

const store = useProjectComponentsStore()
const { run } = useAsyncAction()

const components = computed(() => store.componentsOf(props.projectId))
const componentIds = computed(() => components.value.map((component) => component.id))
const loading = computed(() => store.isLoading(props.projectId))

/** Tiles shown before "Show all": enough to scan, few enough that the sections below stay close. */
const TILES_BEFORE_FOLD = 6

const collapsed = ref(componentsPanelCollapsed())
const showingAll = ref(false)
const editingId = ref<ProjectComponentId | null>(null)
const visibleComponents = computed(() =>
  showingAll.value ? components.value : components.value.slice(0, TILES_BEFORE_FOLD)
)
const hiddenCount = computed(() => components.value.length - visibleComponents.value.length)

const composing = ref(false)
const creating = ref(false)
const deleting = ref<ProjectComponent | null>(null)

function toggleCollapsed(): void {
  collapsed.value = !collapsed.value
  setComponentsPanelCollapsed(collapsed.value)
}

function startComposing(): void {
  collapsed.value = false
  composing.value = true
}

async function add(input: ProjectComponentInput): Promise<void> {
  creating.value = true
  const created = await run(() => store.create(props.projectId, input), 'Component added')
  creating.value = false
  if (created) composing.value = false
}

async function move(component: ProjectComponent, direction: Direction): Promise<void> {
  await run(() => store.move(props.projectId, component.id, direction))
}

async function confirmDelete(): Promise<void> {
  const target = deleting.value
  deleting.value = null
  if (!target) return
  if (editingId.value === target.id) editingId.value = null
  await run(() => store.remove(props.projectId, target.id), `Deleted ${target.name}`)
}

watch(
  () => props.projectId,
  () => {
    composing.value = false
    editingId.value = null
    showingAll.value = false
    void run(() => store.load(props.projectId))
  },
  { immediate: true }
)
</script>

<template>
  <section aria-labelledby="components-heading" data-testid="project-components">
    <div class="flex items-center justify-between gap-4">
      <button
        type="button"
        class="focus-ring -ml-1.5 flex min-w-0 items-center gap-2 rounded-control px-1.5 py-1 text-left"
        :aria-expanded="!collapsed"
        aria-controls="components-body"
        data-testid="components-toggle"
        @click="toggleCollapsed"
      >
        <svg
          class="size-3.5 shrink-0 text-text-subtle transition-transform duration-(--motion-quick)"
          :class="{ '-rotate-90': collapsed }"
          viewBox="0 0 14 14"
          fill="none"
          aria-hidden="true"
        >
          <path d="m3.5 5.5 3.5 3.5 3.5-3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <h2 id="components-heading" class="text-title font-semibold tracking-[-0.01em] text-text">Components</h2>
        <span
          v-if="components.length"
          class="rounded-full bg-surface px-1.5 py-px font-mono text-micro font-normal tabular-nums text-text-subtle"
          data-testid="components-count"
        >
          {{ components.length }}
        </span>
      </button>
      <AppButton
        v-if="!composing"
        :variant="components.length ? 'secondary' : 'primary'"
        size="sm"
        data-testid="add-component"
        @click="startComposing"
      >
        <svg class="size-3" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
        </svg>
        Add component
      </AppButton>
    </div>

    <p v-if="collapsed && components.length" class="mt-1 truncate text-small text-text-subtle" data-testid="components-summary">
      {{ components.map((component) => component.name).join(' · ') }}
    </p>

    <div v-show="!collapsed" id="components-body" class="mt-3 flex flex-col gap-3">
      <p class="-mt-1 text-small text-text-subtle">
        The folders of this project that are parts of it. Every context that loads the project carries their name,
        folder and description.
      </p>

      <Transition name="detail">
        <ProjectComponentComposer
          v-if="composing"
          :root="repoFolder"
          :creating="creating"
          @submit="add"
          @cancel="composing = false"
        />
      </Transition>

      <div
        v-if="loading && !components.length"
        class="grid gap-3 grid-cols-[repeat(auto-fill,minmax(300px,1fr))]"
        role="status"
        aria-live="polite"
        aria-busy="true"
      >
        <span class="sr-only">Loading components</span>
        <div v-for="tile in 3" :key="tile" class="skeleton h-[104px] rounded-card" />
      </div>

      <TransitionGroup v-else tag="ul" name="step-list" class="relative grid gap-3 grid-cols-[repeat(auto-fill,minmax(300px,1fr))]">
        <li v-for="(component, position) in visibleComponents" :key="component.id" :class="{ 'col-span-full': editingId === component.id }">
          <ProjectComponentCard
            v-if="editingId === component.id"
            :component="component"
            :position="position"
            :can-move-up="canMove(componentIds, component.id, 'up')"
            :can-move-down="canMove(componentIds, component.id, 'down')"
            :root="repoFolder"
            @move="move(component, $event)"
            @delete="deleting = component"
            @close="editingId = null"
          />
          <ProjectComponentTile v-else :component="component" :position="position" @open="editingId = component.id" />
        </li>
      </TransitionGroup>

      <div v-if="components.length > TILES_BEFORE_FOLD" class="flex justify-center">
        <AppButton variant="ghost" size="sm" data-testid="components-fold" @click="showingAll = !showingAll">
          {{ showingAll ? 'Show fewer' : `Show all ${components.length} (${hiddenCount} more)` }}
        </AppButton>
      </div>

      <p
        v-if="!loading && !components.length && !composing"
        class="rounded-card border border-dashed border-border px-6 py-8 text-center text-body text-text-subtle"
        data-testid="components-empty"
      >
        A project that is one folder needs none. When it holds several modules, add each subfolder here so a
        session can tell which one you mean.
      </p>
    </div>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        :title="`Delete ${deleting.name}?`"
        body="The component is removed from this project. The folder on disk is not touched."
        :blast="`Removes “${deleting.name}” from the context of ${projectTitle}.`"
        confirm-label="Delete component"
        @cancel="deleting = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </section>
</template>
