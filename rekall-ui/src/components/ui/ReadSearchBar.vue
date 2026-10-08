<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'

/** The find bar over a page being read: the text to find, where the current match is, and the way through them. */
const props = defineProps<{ label: string; count: number; focusRequests: number }>()
const query = defineModel<string>({ required: true })
const emit = defineEmits<{ next: []; previous: []; close: [] }>()

const field = ref<HTMLInputElement | null>(null)

function focusField(): void {
  void nextTick(() => {
    field.value?.focus()
    field.value?.select()
  })
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') emit('close')
  else if (event.key === 'Enter' && event.shiftKey) emit('previous')
  else if (event.key === 'Enter') emit('next')
  else return
  event.preventDefault()
}

onMounted(focusField)
watch(() => props.focusRequests, focusField)
</script>

<template>
  <div
    class="flex items-center gap-1 rounded-card border border-border bg-surface-raised p-1 shadow-lg"
    role="search"
    data-testid="read-search"
  >
    <input
      ref="field"
      v-model="query"
      type="text"
      class="h-7 w-44 rounded-control bg-canvas px-2 text-small text-text outline-none placeholder:text-text-subtle focus:ring-1 focus:ring-accent"
      placeholder="Find in page"
      aria-label="Find in page"
      data-testid="read-search-input"
      @keydown="onKeydown"
    />
    <span
      class="min-w-10 text-center text-caption tabular-nums text-text-subtle"
      :class="{ 'text-danger': query && count === 0 }"
      data-testid="read-search-count"
    >
      {{ label }}
    </span>
    <button
      type="button"
      class="focus-ring flex size-7 items-center justify-center rounded-control text-text-subtle transition-colors hover:text-text disabled:opacity-40"
      aria-label="Previous match"
      :disabled="count === 0"
      data-testid="read-search-previous"
      @click="emit('previous')"
    >
      <svg class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path
          d="M3 7.5 6 4.5l3 3"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <button
      type="button"
      class="focus-ring flex size-7 items-center justify-center rounded-control text-text-subtle transition-colors hover:text-text disabled:opacity-40"
      aria-label="Next match"
      :disabled="count === 0"
      data-testid="read-search-next"
      @click="emit('next')"
    >
      <svg class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path
          d="M3 4.5 6 7.5l3-3"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <button
      type="button"
      class="focus-ring flex size-7 items-center justify-center rounded-control text-text-subtle transition-colors hover:text-text"
      aria-label="Close search"
      data-testid="read-search-close"
      @click="emit('close')"
    >
      <CloseGlyph />
    </button>
  </div>
</template>
