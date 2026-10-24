<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import ProjectComponentCard from '@/features/project/components/ProjectComponentCard.vue'
import ProjectComponentComposer from '@/features/project/components/ProjectComponentComposer.vue'
import ProjectComponentRow from '@/features/project/components/ProjectComponentRow.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { folderName } from '@/features/project/lib/component-folder'
import { componentBranches, componentMatches, type ComponentBranch, type ComponentLeaf } from '@/features/project/lib/component-tree'
import { canMove, type Direction } from '@/features/project/lib/section-order'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import type { ProjectComponent, ProjectComponentInput } from '@/features/project/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

/**
 * The folders a project is made of, drawn as the tree they form under its folder, with the one
 * being edited or added beside it. They ride along in every context that loads the project, so a
 * session told "the billing module" knows which folder that is and what it does.
 */
const props = defineProps<{ projectId: ProjectId; projectTitle: string; repoFolder: string | null }>()

type TreeEntry = { readonly kind: 'leaf'; readonly leaf: ComponentLeaf } | { readonly kind: 'branch'; readonly branch: ComponentBranch }

const store = useProjectComponentsStore()
const { run } = useAsyncAction()

const components = computed(() => store.componentsOf(props.projectId))
const componentIds = computed(() => components.value.map((component) => component.id))
const loading = computed(() => store.isLoading(props.projectId) && !components.value.length)

const query = ref('')
const editingId = ref<ProjectComponentId | null>(null)
const composing = ref(false)
const creating = ref(false)
const deleting = ref<ProjectComponent | null>(null)

const shown = computed(() => components.value.filter((component) => componentMatches(component, query.value)))
const entries = computed<TreeEntry[]>(() =>
  componentBranches(shown.value, props.repoFolder).flatMap((branch): TreeEntry[] =>
    branch.folder === '' ? branch.leaves.map((leaf) => ({ kind: 'leaf', leaf })) : [{ kind: 'branch', branch }]
  )
)
const editing = computed(() => components.value.find((component) => component.id === editingId.value) ?? null)
const editingPosition = computed(() => (editing.value ? componentIds.value.indexOf(editing.value.id) : -1))
const rootName = computed(() => (props.repoFolder ? `${folderName(props.repoFolder)}/` : props.projectTitle))

function startComposing(): void {
  editingId.value = null
  composing.value = true
}

function open(component: ProjectComponent): void {
  composing.value = false
  editingId.value = component.id
}

async function add(input: ProjectComponentInput): Promise<void> {
  creating.value = true
  const created = await run(() => store.create(props.projectId, input), 'Component added')
  creating.value = false
  if (created) composing.value = false
}

async function move(direction: Direction): Promise<void> {
  if (editing.value) await run(() => store.move(props.projectId, editing.value!.id, direction))
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
    query.value = ''
  }
)
</script>

<template>
  <section
    class="flex h-full min-h-0 flex-col overflow-hidden rounded-card border border-border bg-surface shadow-[var(--shadow-raised)]"
    aria-label="Components"
    data-testid="project-components"
  >
    <header class="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-border px-5 py-3">
      <div class="min-w-[240px] flex-1">
        <h2 class="text-title font-semibold tracking-[-0.01em] text-text">Components</h2>
        <p class="mt-0.5 text-small text-text-subtle">
          The folders this project is made of. Every context that loads it lists them, so a session knows which one you mean.
        </p>
      </div>
      <label v-if="components.length" class="sr-only" for="components-filter">Filter components</label>
      <input
        v-if="components.length"
        id="components-filter"
        v-model="query"
        type="search"
        autocomplete="off"
        spellcheck="false"
        placeholder="Filter by name, folder or description"
        class="field text-text h-8 w-[280px] max-w-full rounded-control px-2.5 text-small"
        data-testid="components-filter"
      />
      <AppButton variant="primary" size="sm" data-testid="add-component" @click="startComposing">Add component</AppButton>
    </header>

    <div class="grid min-h-0 flex-1 grid-rows-[minmax(0,1fr)]" :class="{ 'lg:grid-cols-[minmax(0,1fr)_380px]': editing || composing }">
      <div class="min-h-0 overflow-y-auto px-4 py-3">
        <div v-if="loading" class="flex flex-col gap-1" role="status" aria-live="polite" aria-busy="true">
          <span class="sr-only">Loading components</span>
          <div v-for="row in 5" :key="row" class="skeleton h-[34px] rounded-control" />
        </div>

        <div
          v-else-if="!components.length"
          class="mx-auto mt-6 max-w-[52ch] rounded-card border border-dashed border-border px-5 py-5 text-small leading-relaxed text-text-subtle"
          data-testid="components-empty"
        >
          A project that is one folder needs none. When it holds several modules or repositories, add each folder so a
          session can tell which one you mean. <span class="text-text-muted">Find repositories</span>, on the Connections
          tab, adds one for every git repository under the project's folder.
        </div>

        <template v-else>
          <p class="flex h-[30px] items-center gap-2 pl-1 font-mono text-caption text-text-muted" :title="repoFolder ?? undefined" data-testid="components-root">
            <svg class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path d="M2 4.5A1.5 1.5 0 0 1 3.5 3h2.6l1.4 1.5h5A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5v-7Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
            </svg>
            {{ rootName }}
          </p>

          <p v-if="!entries.length" class="pl-7 pt-1 text-small text-text-subtle" data-testid="components-no-match">
            No component matches “{{ query.trim() }}”.
          </p>

          <ul class="branch-list" data-testid="components-tree">
            <li v-for="entry in entries" :key="entry.kind === 'leaf' ? entry.leaf.component.id : `folder:${entry.branch.folder}`">
              <ProjectComponentRow
                v-if="entry.kind === 'leaf'"
                :component="entry.leaf.component"
                :path="entry.leaf.path"
                :open="editingId === entry.leaf.component.id"
                @open="open(entry.leaf.component)"
              />
              <template v-else>
                <p class="flex h-[34px] items-center gap-2 pl-2 font-mono text-caption text-text-muted" data-testid="components-folder">
                  {{ entry.branch.folder ?? 'Outside the project folder' }}
                  <span class="text-text-subtle">{{ entry.branch.leaves.length }}</span>
                </p>
                <ul class="branch-list">
                  <li v-for="leaf in entry.branch.leaves" :key="leaf.component.id">
                    <ProjectComponentRow
                      :component="leaf.component"
                      :path="leaf.path"
                      :open="editingId === leaf.component.id"
                      @open="open(leaf.component)"
                    />
                  </li>
                </ul>
              </template>
            </li>
          </ul>
        </template>
      </div>

      <aside v-if="editing || composing" class="min-h-0 overflow-y-auto border-t border-border p-4 lg:border-l lg:border-t-0" aria-label="Component">
        <ProjectComponentComposer v-if="composing" :root="repoFolder" :creating="creating" @submit="add" @cancel="composing = false" />
        <ProjectComponentCard
          v-else-if="editing"
          :key="editing.id"
          :component="editing"
          :position="editingPosition"
          :can-move-up="canMove(componentIds, editing.id, 'up')"
          :can-move-down="canMove(componentIds, editing.id, 'down')"
          :root="repoFolder"
          @move="move"
          @delete="deleting = editing"
          @close="editingId = null"
        />
      </aside>
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
