<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import ProjectComponentComposer from '@/features/project/components/ProjectComponentComposer.vue'
import ProjectComponentPane from '@/features/project/components/ProjectComponentPane.vue'
import ProjectComponentRail from '@/features/project/components/ProjectComponentRail.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { canMove, type Direction } from '@/features/project/lib/section-order'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectComponent, ProjectComponentInput } from '@/features/project/model/project-component'
import type { ProjectComponentId } from '@/model/branded'

/**
 * The parts a project is made of: an index beside the one open, which is read as a page. A scan
 * hands the folder to a session that writes each part; they appear here as it saves them.
 */
const props = defineProps<{ project: Project; selectedId: ProjectComponentId | null }>()

const emit = defineEmits<{ open: [id: ProjectComponentId | null] }>()

const store = useProjectComponentsStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { run } = useAsyncAction()

const components = computed(() => store.componentsOf(props.project.id))
const componentIds = computed(() => components.value.map((component) => component.id))
const selected = computed(() => components.value.find((component) => component.id === props.selectedId) ?? null)
const loading = computed(() => store.isLoading(props.project.id))
const composing = ref(false)
const creating = ref(false)
const scanning = ref(false)
const deleting = ref<ProjectComponent | null>(null)

async function add(input: ProjectComponentInput): Promise<void> {
  creating.value = true
  const created = await run(() => store.create(props.project.id, input), 'Component added')
  creating.value = false
  if (!created) return
  composing.value = false
  emit('open', created.id)
}

async function scan(): Promise<void> {
  scanning.value = true
  const terminal = await run(() => store.scan(props.project.id))
  scanning.value = false
  if (!terminal) return
  if (terminal.kind === 'TERMINAL') terminals.select(terminal.id)
  toast.notify(`A session is reading ${props.project.title} and writing its components.`)
}

async function move(direction: Direction): Promise<void> {
  if (selected.value) await run(() => store.move(props.project.id, selected.value!.id, direction))
}

async function confirmDelete(): Promise<void> {
  const target = deleting.value
  deleting.value = null
  if (!target) return
  const removed = await run(() => store.remove(props.project.id, target.id).then(() => true), `Deleted ${target.name}`)
  if (removed) emit('open', components.value[0]?.id ?? null)
}

function select(id: ProjectComponentId): void {
  composing.value = false
  emit('open', id)
}

watch(
  () => props.project.id,
  () => {
    composing.value = false
  }
)
</script>

<template>
  <div class="grid h-full min-h-0 grid-cols-1 gap-4 md:grid-cols-[272px_minmax(0,1fr)]" data-testid="project-components">
    <ProjectComponentRail
      class="max-h-[30vh] md:max-h-none"
      :components="components"
      :selected-id="composing ? null : (selected?.id ?? null)"
      :repo-folder="project.repoFolder"
      :loading="loading"
      :scanning="scanning"
      @select="select"
      @scan="scan"
      @add="composing = true"
    />

    <Transition name="fade-quick" mode="out-in">
      <section
        v-if="composing"
        key="composing"
        class="min-h-0 overflow-y-auto rounded-card border border-border bg-surface p-5 shadow-[var(--shadow-raised)]"
        aria-label="New component"
      >
        <h2 class="mb-3 text-title font-semibold text-text">New component</h2>
        <div class="max-w-[560px]">
          <ProjectComponentComposer :root="project.repoFolder" :creating="creating" @submit="add" @cancel="composing = false" />
        </div>
      </section>

      <ProjectComponentPane
        v-else-if="selected"
        :key="selected.id"
        :component="selected"
        :repo-folder="project.repoFolder"
        :can-move-up="canMove(componentIds, selected.id, 'up')"
        :can-move-down="canMove(componentIds, selected.id, 'down')"
        @move="move"
        @delete="deleting = selected"
      />

      <div
        v-else-if="!components.length && !loading"
        key="empty"
        class="grid min-h-[280px] place-items-center rounded-card border border-dashed border-border px-6 py-10"
        data-testid="components-empty"
      >
        <div class="flex max-w-[54ch] flex-col items-start gap-4">
          <h2 class="text-heading font-medium tracking-[-0.01em] text-text">Map what this project is made of</h2>
          <p class="text-body leading-relaxed text-text-muted">
            A session reads the project’s folder and writes a page for each service, module and configuration it finds: what it is
            for, the interfaces it offers, what it depends on, how to run it and where a change goes. A project that is one folder
            needs none.
          </p>
          <div class="flex items-center gap-2">
            <AppButton
              variant="primary"
              :loading="scanning"
              :disabled="!project.repoFolder"
              :title="project.repoFolder ? undefined : 'Set the project folder first'"
              data-testid="scan-components-empty"
              @click="scan"
            >
              Scan folder
            </AppButton>
            <AppButton variant="quiet" @click="composing = true">Add one by hand</AppButton>
          </div>
        </div>
      </div>

      <div
        v-else
        key="none"
        class="grid h-full min-h-[240px] place-items-center rounded-card border border-dashed border-border px-6 text-center"
        data-testid="components-none"
      >
        <p class="max-w-[44ch] text-small leading-relaxed text-text-subtle">Pick a component to read its page.</p>
      </div>
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        :title="`Delete ${deleting.name}?`"
        body="The component and its page are removed from this project. The folder on disk is not touched."
        :blast="`Removes “${deleting.name}” from the context of ${project.title}.`"
        confirm-label="Delete component"
        @cancel="deleting = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </div>
</template>
