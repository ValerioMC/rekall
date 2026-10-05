<script setup lang="ts">
import { nextTick, ref } from 'vue'
import { relativeTime } from '@/common/format/relative-time'
import type { ProjectSection } from '@/model/project-section'
import type { ProjectSectionId } from '@/model/branded'

export type RailSelection = 'blueprint' | ProjectSectionId

/**
 * The project's pages as an index: the blueprint, pinned and lit in the anchor colour because
 * `/rk` hands it to Claude, then the sections, plain because no context loads them.
 */
const props = defineProps<{
  sections: readonly ProjectSection[]
  selected: RailSelection
  loading: boolean
  creating: boolean
}>()

const emit = defineEmits<{ select: [selection: RailSelection]; create: [title: string] }>()

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
  <nav class="flex flex-col gap-1" aria-label="Project pages" data-testid="project-docs-rail">
    <button
      type="button"
      class="focus-ring group flex items-center gap-3 rounded-[var(--radius-control)] py-2.5 pl-4 pr-3 text-left transition-colors"
      :class="selected === 'blueprint' ? 'selected-row' : 'hover:bg-surface-raised'"
      :style="{ '--select-tint': 'var(--color-anchor)', '--select-tint-soft': 'var(--color-anchor-soft)' }"
      :aria-current="selected === 'blueprint' ? 'page' : undefined"
      data-testid="rail-blueprint"
      @click="emit('select', 'blueprint')"
    >
      <svg class="size-4 shrink-0 text-anchor" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <rect x="2.5" y="2.5" width="11" height="11" rx="2" stroke="currentColor" stroke-width="1.3" />
        <path d="M5.5 6h5M5.5 8.5h5M5.5 11h2.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      </svg>
      <span class="min-w-0 flex-1">
        <span class="block truncate text-[13px] font-medium text-text">Blueprint</span>
        <span class="block truncate text-[11.5px] text-anchor/80">Loaded by every /rk</span>
      </span>
    </button>

    <div class="mt-3 flex items-center justify-between pl-4 pr-1">
      <p class="eyebrow">Sections<span v-if="sections.length" class="ml-1.5 text-text-subtle/80">{{ sections.length }}</span></p>
      <button
        v-if="!composing"
        type="button"
        class="key-quiet focus-ring grid size-6 place-items-center rounded-[3px]"
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
          class="field text-text h-(--spacing-control) w-full rounded-[var(--radius-control)] px-3 text-[13px]"
          data-testid="rail-new-title"
          @keydown.esc.stop.prevent="stopComposing"
          @blur="title.trim() === '' && stopComposing()"
        />
      </form>
    </Transition>

    <div v-if="loading && !sections.length" class="flex flex-col gap-1.5 px-1 pt-1" role="status" aria-live="polite" aria-busy="true">
      <span class="sr-only">Loading sections</span>
      <div v-for="row in 2" :key="row" class="skeleton h-(--spacing-row) rounded-[var(--radius-control)]" />
    </div>

    <TransitionGroup v-else tag="ul" name="step-list" class="relative flex flex-col gap-1">
      <li v-for="section in sections" :key="section.id">
        <button
          type="button"
          class="focus-ring flex w-full items-center rounded-[var(--radius-control)] py-2 pl-4 pr-3 text-left transition-colors"
          :class="selected === section.id ? 'selected-row' : 'hover:bg-surface-raised'"
          :aria-current="selected === section.id ? 'page' : undefined"
          :data-testid="`rail-section-${section.id}`"
          @click="emit('select', section.id)"
        >
          <span class="min-w-0 flex-1">
            <span class="block truncate text-[13px] text-text">{{ section.title }}</span>
            <span class="block truncate text-[11.5px] text-text-subtle">
              {{ section.bodyMarkdown.trim() === '' ? 'Empty' : `Edited ${relativeTime(section.updatedAt)}` }}
            </span>
          </span>
        </button>
      </li>
    </TransitionGroup>

    <p
      v-if="!loading && !sections.length && !composing"
      class="px-4 pt-1 text-[12px] leading-relaxed text-text-subtle"
      data-testid="rail-empty"
    >
      Keep what the blueprint doesn't hold: deployment, conventions, a glossary.
    </p>

    <p class="mt-3 border-t border-border px-4 pt-3 text-[11.5px] leading-relaxed text-text-subtle">
      Sections stay out of Claude's context. A session adds one with
      <code class="text-text-muted">rekall_project_section</code>.
    </p>
  </nav>
</template>
