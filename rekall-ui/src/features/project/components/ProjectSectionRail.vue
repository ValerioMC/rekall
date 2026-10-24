<script setup lang="ts">
import { nextTick, ref } from 'vue'
import { relativeTime } from '@/common/format/relative-time'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { ProjectSectionId } from '@/model/branded'

/** The project's sections as an index beside the one open, with a field to start another. */
const props = defineProps<{
  sections: readonly ProjectSection[]
  selectedId: ProjectSectionId | null
  loading: boolean
  creating: boolean
}>()

const emit = defineEmits<{ select: [id: ProjectSectionId]; create: [title: string] }>()

const composing = ref(false)
const title = ref('')
const titleInput = ref<HTMLInputElement | null>(null)

async function startComposing(): Promise<void> {
  composing.value = true
  await nextTick()
  titleInput.value?.focus()
}

function stopComposing(): void {
  composing.value = false
  title.value = ''
}

function submit(): void {
  const wanted = title.value.trim()
  if (!wanted || props.creating) return
  emit('create', wanted)
  stopComposing()
}
</script>

<template>
  <nav class="flex min-h-0 flex-col gap-1 overflow-y-auto" aria-label="Sections" data-testid="project-section-rail">
    <div class="flex items-center justify-between pl-4 pr-1">
      <p class="eyebrow">Sections<span v-if="sections.length" class="ml-1.5 text-text-subtle">{{ sections.length }}</span></p>
      <button
        v-if="!composing"
        type="button"
        class="key-quiet focus-ring grid size-6 place-items-center rounded-key"
        aria-label="New section"
        title="New section"
        data-testid="rail-new-section"
        @click="startComposing"
      >
        <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <Transition name="detail">
      <form v-if="composing" class="px-1" data-testid="rail-new-form" @submit.prevent="submit">
        <label class="sr-only" for="new-section-title">Section title</label>
        <input
          id="new-section-title"
          ref="titleInput"
          v-model="title"
          maxlength="200"
          autocomplete="off"
          placeholder="Title, then Enter"
          class="field text-text h-(--spacing-control) w-full rounded-control px-3 text-body"
          data-testid="rail-new-title"
          @keydown.esc.stop.prevent="stopComposing"
          @blur="title.trim() === '' && stopComposing()"
        />
      </form>
    </Transition>

    <div v-if="loading && !sections.length" class="flex flex-col gap-1.5 px-1 pt-1" role="status" aria-live="polite" aria-busy="true">
      <span class="sr-only">Loading sections</span>
      <div v-for="row in 2" :key="row" class="skeleton h-(--spacing-row) rounded-control" />
    </div>

    <TransitionGroup v-else tag="ul" name="step-list" class="relative flex flex-col gap-1">
      <li v-for="section in sections" :key="section.id">
        <button
          type="button"
          class="focus-ring flex w-full items-center rounded-control py-2 pl-4 pr-3 text-left transition-colors"
          :class="selectedId === section.id ? 'selected-row' : 'hover:bg-surface-raised'"
          :aria-current="selectedId === section.id ? 'page' : undefined"
          :data-testid="`rail-section-${section.id}`"
          @click="emit('select', section.id)"
        >
          <span class="min-w-0 flex-1">
            <span class="block truncate text-body text-text">{{ section.title }}</span>
            <span class="block truncate text-caption text-text-subtle">
              {{ section.bodyMarkdown.trim() === '' ? 'Empty' : `Edited ${relativeTime(section.updatedAt)}` }}
            </span>
          </span>
        </button>
      </li>
    </TransitionGroup>

    <p
      v-if="!loading && !sections.length && !composing"
      class="px-4 pt-1 text-small leading-relaxed text-text-subtle"
      data-testid="rail-empty"
    >
      Keep what the blueprint doesn't hold: deployment, conventions, a glossary.
    </p>

    <p class="mt-auto border-t border-border px-4 pt-3 text-caption leading-relaxed text-text-subtle">
      Sections stay out of Claude's context. A session adds one with
      <code class="text-text-muted">rekall_project_section</code>.
    </p>
  </nav>
</template>
