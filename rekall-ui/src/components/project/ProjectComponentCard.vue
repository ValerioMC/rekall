<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import FolderPickerButton from '@/components/project/FolderPickerButton.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { useProjectComponentsStore } from '@/stores/projectComponents.store'
import type { ProjectComponent, ProjectComponentInput } from '@/model/project-component'

/**
 * One component, edited in place of its row, laid out for the inspector's narrow column. Each field saves on its own: a pause in typing or
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
  <div
    class="editor relative rounded-control border border-border-strong bg-canvas p-3 pl-3.5"
    :style="{ '--component-hue': hue.base, '--component-hue-line': hue.line }"
    :data-testid="`component-${component.id}`"
  >
    <span class="component-rail" aria-hidden="true" />

    <div class="flex items-center gap-1.5">
      <span class="component-index shrink-0 font-mono text-micro tabular-nums" aria-hidden="true">
        {{ String(position + 1).padStart(2, '0') }}
      </span>
      <label class="sr-only" :for="`component-name-${component.id}`">Component name</label>
      <input
        :id="`component-name-${component.id}`"
        v-model="nameDraft"
        class="quiet-field min-w-0 flex-1 text-body font-semibold text-text"
        :aria-invalid="rejected || undefined"
        maxlength="200"
        autocomplete="off"
        spellcheck="false"
        data-testid="component-name"
        @input="edited"
        @blur="save"
      />
    </div>

    <label class="sr-only" :for="`component-folder-${component.id}`">Component folder</label>
    <div class="mt-2 flex items-center gap-1.5">
      <input
        :id="`component-folder-${component.id}`"
        v-model="folderDraft"
        class="field text-text h-8 min-w-0 flex-1 rounded-control px-2 font-mono text-caption"
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
      class="field text-text mt-2 block min-h-[52px] w-full resize-none overflow-hidden rounded-control p-2 text-small leading-relaxed"
      placeholder="What this component is and what it owns, so a session knows when you mean it."
      data-testid="component-description"
      @input="edited"
      @blur="save"
    />

    <div class="mt-2 flex items-center gap-0.5">
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
      <span class="flex-1" />
      <AppButton variant="danger-quiet" size="sm" aria-label="Delete component" data-testid="component-delete" @click="emit('delete')">
        Delete
      </AppButton>
      <AppButton variant="secondary" size="sm" data-testid="component-close" @click="close">Done</AppButton>
    </div>
  </div>
</template>

<style scoped>
/* The component's own hue runs down the editor's edge, the same mark its row carries. */
.component-rail {
  position: absolute;
  inset: 10px auto 10px -1px;
  width: 2px;
  border-radius: 0 2px 2px 0;
  background: var(--component-hue);
  box-shadow: 0 0 12px -2px var(--component-hue-line);
}

.component-index {
  color: var(--component-hue);
  letter-spacing: 0.04em;
}

/* Reads as a title until touched: no border at rest, the field's own recess on hover and focus. */
.quiet-field {
  height: 28px;
  border: 1px solid transparent;
  border-radius: var(--radius-control);
  background: transparent;
  padding: 0 6px;
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
  background: var(--color-surface);
  border-color: color-mix(in srgb, var(--color-accent) 62%, transparent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-accent) 14%, transparent);
}

.quiet-field[aria-invalid='true'] {
  border-color: color-mix(in srgb, var(--color-danger) 70%, transparent);
}
</style>
