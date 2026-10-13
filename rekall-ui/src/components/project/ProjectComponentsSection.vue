<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import ProjectComponentCard from '@/components/project/ProjectComponentCard.vue'
import ProjectComponentComposer from '@/components/project/ProjectComponentComposer.vue'
import ProjectComponentRow from '@/components/project/ProjectComponentRow.vue'
import InspectorGroup from '@/components/project/InspectorGroup.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { canMove, type Direction } from '@/common/project/section-order'
import { useProjectComponentsStore } from '@/stores/projectComponents.store'
import type { ProjectComponent, ProjectComponentInput } from '@/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

/**
 * The folders of a project that are parts of it, as a dense list in the inspector. They ride along
 * in every context that loads the project, so a session told "the billing module" knows which
 * folder that is and what it does.
 */
const props = defineProps<{ projectId: ProjectId; projectTitle: string; repoFolder: string | null }>()

const store = useProjectComponentsStore()
const { run } = useAsyncAction()

const components = computed(() => store.componentsOf(props.projectId))
const componentIds = computed(() => components.value.map((component) => component.id))
const loading = computed(() => store.isLoading(props.projectId))

const group = ref<InstanceType<typeof InspectorGroup> | null>(null)
const editingId = ref<ProjectComponentId | null>(null)
const summary = computed(() => components.value.map((component) => component.name).join(' · '))

const composing = ref(false)
const creating = ref(false)
const deleting = ref<ProjectComponent | null>(null)

function startComposing(): void {
  group.value?.expand()
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
    void run(() => store.load(props.projectId))
  },
  { immediate: true }
)
</script>

<template>
  <InspectorGroup ref="group" group="components" title="Components" :count="components.length" :summary="summary" data-testid="project-components">
    <template #actions>
      <button
        v-if="!composing"
        type="button"
        class="key-quiet focus-ring grid size-6 cursor-pointer place-items-center rounded-key"
        aria-label="Add component"
        title="Add component"
        data-testid="add-component"
        @click="startComposing"
      >
        <svg class="size-3" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
        </svg>
      </button>
    </template>

    <div class="flex flex-col gap-1.5">
      <Transition name="detail">
        <ProjectComponentComposer
          v-if="composing"
          :root="repoFolder"
          :creating="creating"
          @submit="add"
          @cancel="composing = false"
        />
      </Transition>

      <div v-if="loading && !components.length" class="flex flex-col gap-1" role="status" aria-live="polite" aria-busy="true">
        <span class="sr-only">Loading components</span>
        <div v-for="row in 3" :key="row" class="skeleton h-[52px] rounded-control" />
      </div>

      <TransitionGroup v-else tag="ul" name="step-list" class="relative -mx-1.5 flex flex-col gap-0.5">
        <li v-for="(component, position) in components" :key="component.id">
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
          <ProjectComponentRow v-else :component="component" :position="position" @open="editingId = component.id" />
        </li>
      </TransitionGroup>

      <p
        v-if="!loading && !components.length && !composing"
        class="rounded-control border border-dashed border-border px-3 py-3 text-caption leading-relaxed text-text-subtle"
        data-testid="components-empty"
      >
        A project that is one folder needs none. When it holds several modules, add each subfolder so a session can
        tell which one you mean.
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
  </InspectorGroup>
</template>
