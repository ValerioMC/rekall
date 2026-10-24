<script setup lang="ts">
import { computed, ref } from 'vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import ProjectSectionPane from '@/features/project/components/ProjectSectionPane.vue'
import ProjectSectionRail from '@/features/project/components/ProjectSectionRail.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { canMove, type Direction } from '@/features/project/lib/section-order'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { ProjectSectionId } from '@/model/branded'

/** The project's sections: their index beside the one open, written, moved and deleted in place. */
const props = defineProps<{ project: Project; selectedId: ProjectSectionId | null }>()

const emit = defineEmits<{ open: [id: ProjectSectionId | null] }>()

const store = useProjectSectionsStore()
const { run } = useAsyncAction()

const sections = computed(() => store.sectionsOf(props.project.id))
const sectionIds = computed(() => sections.value.map((section) => section.id))
const selected = computed(() => sections.value.find((section) => section.id === props.selectedId) ?? null)
const creating = ref(false)
const deleting = ref<ProjectSection | null>(null)

async function create(title: string): Promise<void> {
  creating.value = true
  const created = await run(() => store.create(props.project.id, title))
  creating.value = false
  if (created) emit('open', created.id)
}

async function move(direction: Direction): Promise<void> {
  if (selected.value) await run(() => store.move(props.project.id, selected.value!.id, direction))
}

async function confirmDelete(): Promise<void> {
  const target = deleting.value
  deleting.value = null
  if (!target) return
  const removed = await run(() => store.remove(props.project.id, target.id).then(() => true), `Deleted ${target.title}`)
  if (removed) emit('open', sections.value[0]?.id ?? null)
}
</script>

<template>
  <div class="grid h-full min-h-0 grid-cols-1 gap-4 md:grid-cols-[232px_minmax(0,1fr)]" data-testid="sections-page">
    <ProjectSectionRail
      class="max-h-[30vh] md:max-h-none"
      :sections="sections"
      :selected-id="selected?.id ?? null"
      :loading="store.isLoading(project.id)"
      :creating="creating"
      @select="emit('open', $event)"
      @create="create"
    />

    <Transition name="fade-quick" mode="out-in">
      <ProjectSectionPane
        v-if="selected"
        :key="selected.id"
        :section="selected"
        :repo-folder="project.repoFolder"
        :can-move-up="canMove(sectionIds, selected.id, 'up')"
        :can-move-down="canMove(sectionIds, selected.id, 'down')"
        @move="move"
        @delete="deleting = selected"
      />
      <div
        v-else
        key="none"
        class="grid h-full min-h-[240px] place-items-center rounded-card border border-dashed border-border px-6 text-center"
        data-testid="sections-none"
      >
        <p class="max-w-[44ch] text-small leading-relaxed text-text-subtle">
          {{ sections.length ? 'Pick a section to read or write it.' : 'Start one with + to keep what the blueprint does not hold.' }}
        </p>
      </div>
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        :title="`Delete ${deleting.title}?`"
        body="The section and everything written in it is removed. No history keeps it."
        :blast="`Deletes the section “${deleting.title}” from ${project.title}.`"
        confirm-label="Delete section"
        @cancel="deleting = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </div>
</template>
