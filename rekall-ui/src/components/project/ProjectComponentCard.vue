<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import FolderPickerButton from '@/components/project/FolderPickerButton.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { useProjectComponentsStore } from '@/stores/projectComponents.store'
import type { ProjectComponent, ProjectComponentInput } from '@/model/project-component'

/**
 * One component, edited where it is read. Each field saves on its own: a pause in typing or
 * leaving the field. A save the server refuses (a name already taken) keeps the draft on screen,
 * marked invalid, until the next keystroke, so what was typed is never silently put back.
 */
const props = defineProps<{
  component: ProjectComponent
  position: number
  canMoveUp: boolean
  canMoveDown: boolean
  root: string | null
}>()

const emit = defineEmits<{ move: [direction: 'up' | 'down']; delete: []; close: [] }>()

const SAVE_DELAY_MS = 700

const store = useProjectComponentsStore()
const { run } = useAsyncAction()

const nameDraft = ref(props.component.name)
const folderDraft = ref(props.component.folder)
const descriptionDraft = ref(props.component.description)
const rejected = ref(false)
const descriptionArea = ref<HTMLTextAreaElement | null>(null)
let timer: ReturnType<typeof setTimeout> | null = null

const hue = computed(() => identityHue(props.component.id))

/** What differs from the saved component; a name or folder cannot be saved blank. */
function pendingChange(): ProjectComponentInput | null {
  const name = nameDraft.value.trim()
  const folderPath = folderDraft.value.trim()
  const description = descriptionDraft.value.trim()
  const change: { name?: string; folder?: string; description?: string } = {}
  if (name !== '' && name !== props.component.name) change.name = name
  if (folderPath !== '' && folderPath !== props.component.folder) change.folder = folderPath
  if (description !== props.component.description) change.description = description
  return Object.keys(change).length > 0 ? change : null
}

function cancelSchedule(): void {
  if (timer) clearTimeout(timer)
  timer = null
}

async function save(): Promise<void> {
  cancelSchedule()
  const change = pendingChange()
  if (!change) return
  rejected.value = (await run(() => store.update(props.component.id, change))) === null
}

function edited(): void {
  rejected.value = false
  cancelSchedule()
  timer = setTimeout(() => void save(), SAVE_DELAY_MS)
}

function grow(): void {
  const area = descriptionArea.value
  if (!area) return
  area.style.height = 'auto'
  area.style.height = `${area.scrollHeight}px`
}

async function close(): Promise<void> {
  await save()
  if (!rejected.value) emit('close')
}

function folderChosen(chosen: string): void {
  folderDraft.value = chosen
  edited()
}

// A saved value that arrives from outside (another save, a reload) replaces a draft that was not touched.
watch(
  () => props.component,
  (saved, before) => {
    if (nameDraft.value.trim() === before.name) nameDraft.value = saved.name
    if (folderDraft.value.trim() === before.folder) folderDraft.value = saved.folder
    if (descriptionDraft.value.trim() === before.description) descriptionDraft.value = saved.description
  }
)
watch(descriptionDraft, () => void nextTick(grow))
onMounted(grow)
onBeforeUnmount(() => {
  const stillListed = store.componentsOf(props.component.projectId).some((known) => known.id === props.component.id)
  if (timer && stillListed) void save()
  else cancelSchedule()
})
</script>

<template>
  <AppCard
    :padded="false"
    class="group relative flex gap-4 p-4 pl-5"
    :style="{ '--component-hue': hue.base, '--component-hue-soft': hue.soft, '--component-hue-line': hue.line }"
    :data-testid="`component-${component.id}`"
  >
    <span class="component-rail" aria-hidden="true" />

    <span class="component-index mt-0.5 shrink-0 font-mono text-caption tabular-nums" aria-hidden="true">
      {{ String(position + 1).padStart(2, '0') }}
    </span>

    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <label class="sr-only" :for="`component-name-${component.id}`">Component name</label>
        <input
          :id="`component-name-${component.id}`"
          v-model="nameDraft"
          class="quiet-field min-w-0 flex-1 text-title font-semibold tracking-[-0.01em] text-text"
          :aria-invalid="rejected || undefined"
          maxlength="200"
          autocomplete="off"
          spellcheck="false"
          data-testid="component-name"
          @input="edited"
          @blur="save"
        />
        <div class="flex shrink-0 items-center gap-0.5 opacity-60 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100">
          <AppButton
            variant="quiet"
            size="icon"
            class="disabled:opacity-30"
            :disabled="!canMoveUp"
            aria-label="Move up"
            title="Move up"
            data-testid="component-up"
            @click="emit('move', 'up')"
          >
            <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
              <path d="M3.5 8.5 7 5l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </AppButton>
          <AppButton
            variant="quiet"
            size="icon"
            class="disabled:opacity-30"
            :disabled="!canMoveDown"
            aria-label="Move down"
            title="Move down"
            data-testid="component-down"
            @click="emit('move', 'down')"
          >
            <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
              <path d="M3.5 5.5 7 9l3.5-3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </AppButton>
          <AppButton variant="danger-quiet" size="sm" aria-label="Delete component" data-testid="component-delete" @click="emit('delete')">
            Delete
          </AppButton>
          <AppButton variant="secondary" size="sm" data-testid="component-close" @click="close">Done</AppButton>
        </div>
      </div>

      <div class="mt-2 flex items-center gap-2">
        <svg class="size-4 shrink-0 text-[var(--component-hue)]" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M2 4.5A1.5 1.5 0 0 1 3.5 3h2.6l1.4 1.5h5A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5v-7Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
        <label class="sr-only" :for="`component-folder-${component.id}`">Component folder</label>
        <input
          :id="`component-folder-${component.id}`"
          v-model="folderDraft"
          class="field text-text h-8 min-w-0 flex-1 rounded-control px-2.5 font-mono text-small"
          :aria-invalid="rejected || undefined"
          maxlength="1000"
          autocomplete="off"
          spellcheck="false"
          placeholder="/path/to/the/component"
          data-testid="component-folder"
          @input="edited"
          @blur="save"
        />
        <FolderPickerButton :root="root" :current="folderDraft" data-testid="component-browse" @pick="folderChosen" />
      </div>

      <label class="sr-only" :for="`component-description-${component.id}`">What this component is</label>
      <textarea
        :id="`component-description-${component.id}`"
        ref="descriptionArea"
        v-model="descriptionDraft"
        rows="2"
        class="field text-text mt-3 block min-h-[56px] w-full resize-none overflow-hidden rounded-control p-3 text-body leading-relaxed"
        placeholder="What this component is and what it owns, so a session knows when you mean it."
        data-testid="component-description"
        @input="edited"
        @blur="save"
      />
    </div>
  </AppCard>
</template>

<style scoped>
/* The component's own hue runs down the card's edge: the one thing that tells two cards apart at a glance. */
.component-rail {
  position: absolute;
  inset: 12px auto 12px 0;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: var(--component-hue);
  box-shadow: 0 0 14px -2px var(--component-hue-line);
}

.component-index {
  color: var(--component-hue);
  letter-spacing: 0.04em;
}

/* Reads as a title until touched: no border at rest, the field's own recess on hover and focus. */
.quiet-field {
  height: 30px;
  border: 1px solid transparent;
  border-radius: var(--radius-control);
  background: transparent;
  padding: 0 8px;
  margin-left: -8px;
  transition:
    border-color var(--motion-quick) ease,
    background-color var(--motion-quick) ease,
    box-shadow var(--motion-quick) ease;
}

.quiet-field:hover {
  border-color: var(--color-border);
}

.quiet-field:focus {
  outline: none;
  background: var(--color-canvas);
  border-color: color-mix(in srgb, var(--color-accent) 62%, transparent);
  box-shadow:
    inset 0 1px 2px rgb(0 0 0 / 0.38),
    0 0 0 3px color-mix(in srgb, var(--color-accent) 14%, transparent);
}

.quiet-field[aria-invalid='true'] {
  border-color: color-mix(in srgb, var(--color-danger) 70%, transparent);
}
</style>
