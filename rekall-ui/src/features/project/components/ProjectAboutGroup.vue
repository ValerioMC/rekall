<script setup lang="ts">
import { nextTick, onUnmounted, ref, watch } from 'vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

/** The project's short description, saved after a pause in typing. It travels into every context that loads the project. */
const props = defineProps<{ project: Project }>()

const SAVE_DELAY_MS = 700

const store = useConsoleStore()
const draft = ref('')
const area = ref<HTMLTextAreaElement | null>(null)
let timer: ReturnType<typeof setTimeout> | null = null

function grow(): void {
  if (!area.value) return
  area.value.style.height = 'auto'
  area.value.style.height = `${area.value.scrollHeight}px`
}

function save(id: ProjectId): void {
  if (timer) clearTimeout(timer)
  timer = null
  void store.saveProjectDescription(id, draft.value)
}

function onInput(event: Event): void {
  draft.value = (event.target as HTMLTextAreaElement).value
  grow()
  if (timer) clearTimeout(timer)
  const id = props.project.id
  timer = setTimeout(() => save(id), SAVE_DELAY_MS)
}

watch(
  () => props.project.id,
  async (_, previous) => {
    if (timer && previous) save(previous)
    draft.value = props.project.description ?? ''
    await nextTick()
    grow()
  },
  { immediate: true }
)

onUnmounted(() => {
  if (timer) save(props.project.id)
})
</script>

<template>
  <section aria-labelledby="project-description-heading" data-testid="settings-about">
    <h3 id="project-description-heading" class="text-body font-semibold text-text">Description</h3>
    <p class="mb-2 mt-0.5 text-small text-text-subtle">A few sentences on what this is. It travels into every context that loads the project.</p>
    <label class="sr-only" for="project-description">Project description</label>
    <textarea
      id="project-description"
      ref="area"
      :value="draft"
      data-testid="project-description"
      rows="3"
      class="field text-text block max-h-[280px] w-full resize-none overflow-y-auto rounded-control p-2.5 text-small leading-relaxed"
      placeholder="What this is, in a few sentences."
      @input="onInput"
    />
  </section>
</template>
