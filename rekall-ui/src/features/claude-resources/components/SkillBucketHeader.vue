<script setup lang="ts">
import { nextTick, ref } from 'vue'

/** A group of the person's skills: folds the group, and renames it in Rekall. */
const props = defineProps<{ label: string; count: number; expanded: boolean; bodyId: string }>()

const emit = defineEmits<{ toggle: []; rename: [name: string] }>()

const renaming = ref(false)
const draft = ref('')
const field = ref<HTMLInputElement | null>(null)

async function startRenaming(): Promise<void> {
  draft.value = props.label
  renaming.value = true
  await nextTick()
  field.value?.select()
}

function commit(): void {
  const name = draft.value.trim()
  renaming.value = false
  if (name !== '' && name !== props.label) emit('rename', name)
}
</script>

<template>
  <div class="group/header relative flex items-center">
    <form v-if="renaming" class="flex min-w-0 flex-1 items-center gap-1 px-2 py-1" @submit.prevent="commit">
      <label class="sr-only" :for="`${bodyId}-rename`">Rename {{ label }}</label>
      <input
        :id="`${bodyId}-rename`"
        ref="field"
        v-model="draft"
        maxlength="60"
        autocomplete="off"
        spellcheck="false"
        class="field h-7 min-w-0 flex-1 rounded-control px-2 text-small text-text"
        data-testid="skill-bucket-rename"
        @keydown.esc.stop="renaming = false"
        @blur="commit"
      />
    </form>
    <template v-else>
      <button
        type="button"
        class="focus-ring flex min-w-0 flex-1 cursor-pointer items-center gap-2 rounded-control px-2 py-1 text-left transition-colors hover:bg-surface-raised"
        :aria-expanded="expanded"
        :aria-controls="bodyId"
        @click="emit('toggle')"
      >
        <svg class="size-3 shrink-0 text-text-subtle transition-transform motion-reduce:transition-none" :class="expanded ? 'rotate-90' : ''" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <svg class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M2 4.5A1.5 1.5 0 0 1 3.5 3h2.8l1.4 1.5h4.8A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
        <span class="min-w-0 flex-1 truncate text-small font-semibold text-text">{{ label }}</span>
        <span class="shrink-0 font-mono text-micro tabular-nums text-text-subtle">{{ count }}</span>
      </button>
      <button
        type="button"
        class="focus-ring absolute right-8 grid size-6 cursor-pointer place-items-center rounded-key text-text-subtle opacity-0 transition-opacity hover:bg-surface hover:text-text focus-visible:opacity-100 group-hover/header:opacity-100"
        :aria-label="`Rename ${label}`"
        title="Rename this group (Rekall only)"
        data-testid="skill-bucket-rename-button"
        @click="startRenaming"
      >
        <svg class="size-3" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="m11 2.5 2.5 2.5L6 12.5l-3.2.7.7-3.2z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
      </button>
    </template>
  </div>
</template>
