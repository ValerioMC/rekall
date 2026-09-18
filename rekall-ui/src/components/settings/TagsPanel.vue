<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import TagIcon from '@/components/ui/TagIcon.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useModalGate } from '@/composables/useModalGate'
import { useToastStore } from '@/stores/toast.store'
import { trapTabKey } from '@/common/a11y/focus-trap'
import {
  TAG_COLOR_KEYS,
  TAG_COLOR_LABEL,
  TAG_ICON_KEYS,
  TAG_ICON_LABEL,
  type TagColorKey,
  type TagIconKey
} from '@/model/tag'
import type { Tag } from '@/model/catalog'
import type { TagId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const toast = useToastStore()
const { open: openModal, close: closeModal } = useModalGate()

const panel = ref<HTMLElement | null>(null)
const closeButton = ref<HTMLButtonElement | null>(null)
const nameField = ref<HTMLInputElement | null>(null)

const editingId = ref<TagId | null>(null)
const isNew = ref(false)
const name = ref('')
const icon = ref<TagIconKey>('star')
const color = ref<TagColorKey>('crimson')
const submitting = ref(false)
const error = ref<string | null>(null)
const deleting = ref<Tag | null>(null)

const formOpen = computed(() => isNew.value || editingId.value !== null)

function openCreate(): void {
  editingId.value = null
  isNew.value = true
  name.value = ''
  icon.value = 'star'
  color.value = 'crimson'
  error.value = null
  focusName()
}

function openEdit(tag: Tag): void {
  isNew.value = false
  editingId.value = tag.id
  name.value = tag.name
  icon.value = TAG_ICON_KEYS.includes(tag.icon as TagIconKey) ? (tag.icon as TagIconKey) : 'star'
  color.value = TAG_COLOR_KEYS.includes(tag.color as TagColorKey) ? (tag.color as TagColorKey) : 'crimson'
  error.value = null
  focusName()
}

async function focusName(): Promise<void> {
  await nextTick()
  nameField.value?.focus()
  nameField.value?.select()
}

function closeForm(): void {
  isNew.value = false
  editingId.value = null
  error.value = null
}

async function save(): Promise<void> {
  if (!name.value.trim()) {
    error.value = 'A name is required.'
    return
  }
  submitting.value = true
  error.value = null
  try {
    const input = { name: name.value.trim(), icon: icon.value, color: color.value }
    if (editingId.value) await store.updateTag(editingId.value, input)
    else await store.createTag(input)
    closeForm()
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : 'Could not save the tag.'
  } finally {
    submitting.value = false
  }
}

async function confirmDelete(): Promise<void> {
  if (!deleting.value) return
  try {
    await store.deleteTag(deleting.value.id)
    toast.notify(`Deleted ${deleting.value.name}`)
  } catch (caught) {
    toast.notifyError(caught)
  } finally {
    deleting.value = null
  }
}

function taskCount(tagId: TagId): number {
  return store.tasks.filter((task) => task.tagId === tagId).length
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    if (deleting.value) return
    if (formOpen.value) closeForm()
    else emit('close')
    return
  }
  if (panel.value && !deleting.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  await store.refreshTags()
  await nextTick()
  closeButton.value?.focus()
})
onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    @click.self="!deleting && emit('close')"
  >
    <div
      ref="panel"
      class="dialog-panel flex max-h-[85vh] w-full max-w-[560px] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
      role="dialog"
      aria-modal="true"
      aria-label="Tags"
      data-testid="tags-panel"
    >
      <header class="flex items-center gap-3 border-b border-border px-6 py-4">
        <span class="text-[15px] font-semibold tracking-[-0.01em] text-text">Tags</span>
        <button
          ref="closeButton"
          type="button"
          class="focus-ring ml-auto grid size-7 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
          aria-label="Close"
          @click="emit('close')"
        >
          &times;
        </button>
      </header>

      <div class="min-h-0 flex-1 overflow-y-auto px-6 py-5">
        <p class="mb-3 eyebrow text-[11px]">
          Configured tags
        </p>
        <p class="mb-4 text-[12.5px] leading-relaxed text-text-subtle">
          A tag is a name, a glowing icon and a glow colour. Assign one to a task from the task's
          edit dialog; it shows as a badge wherever that task is listed.
        </p>

        <ul v-if="store.tags.length" class="space-y-2">
          <li
            v-for="tag in store.tags"
            :key="tag.id"
            class="flex items-center gap-3 rounded-[var(--radius-control)] border border-border bg-canvas px-3.5 py-3"
            data-testid="tag-row"
          >
            <TagBadge size="md" :name="tag.name" :icon="tag.icon" :color="tag.color" />
            <span class="ml-auto shrink-0 text-[11.5px] text-text-subtle">
              {{ taskCount(tag.id) }} task{{ taskCount(tag.id) === 1 ? '' : 's' }}
            </span>
            <AppButton size="sm" variant="ghost" @click="openEdit(tag)">Edit</AppButton>
            <AppButton size="sm" variant="danger" @click="deleting = tag">Delete</AppButton>
          </li>
        </ul>
        <p v-else class="text-[12.5px] text-text-subtle">
          No tags yet. Add the first one below.
        </p>

        <div class="mt-4">
          <AppButton v-if="!formOpen" variant="ghost" size="sm" @click="openCreate">
            + Add a tag
          </AppButton>

          <div
            v-else
            class="rounded-[var(--radius-control)] border border-border-strong bg-canvas p-4"
            data-testid="tag-form"
          >
            <label for="tag-name" class="mb-1.5 block eyebrow text-[11px]">Name</label>
            <input
              id="tag-name"
              ref="nameField"
              v-model="name"
              data-testid="tag-name"
              class="focus-ring h-9 w-full rounded-[var(--radius-control)] border border-border bg-surface px-3 text-[13.5px] text-text outline-none transition-colors placeholder:text-text-subtle hover:border-border-strong focus:border-accent"
              placeholder="Bug, Idea, Blocked…"
              @keydown.enter="save"
            />

            <p class="mb-1.5 mt-4 eyebrow text-[11px]">Icon</p>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="option in TAG_ICON_KEYS"
                :key="option"
                type="button"
                class="focus-ring grid size-9 place-items-center rounded-[var(--radius-control)] border transition-colors"
                :class="
                  icon === option
                    ? 'border-accent bg-accent-soft'
                    : 'border-border-strong bg-surface hover:border-text-subtle'
                "
                :aria-pressed="icon === option"
                :aria-label="TAG_ICON_LABEL[option]"
                :title="TAG_ICON_LABEL[option]"
                @click="icon = option"
              >
                <TagIcon :icon="option" :color="color" :size="16" />
              </button>
            </div>

            <p class="mb-1.5 mt-4 eyebrow text-[11px]">Glow colour</p>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="option in TAG_COLOR_KEYS"
                :key="option"
                type="button"
                class="focus-ring grid size-9 place-items-center rounded-full border-2 transition-transform"
                :class="color === option ? 'scale-110 border-text' : 'border-transparent hover:scale-105'"
                :style="{
                  backgroundColor: `var(--color-tag-${option})`,
                  boxShadow: `0 0 8px -1px var(--color-tag-${option})`
                }"
                :aria-pressed="color === option"
                :aria-label="TAG_COLOR_LABEL[option]"
                :title="TAG_COLOR_LABEL[option]"
                @click="color = option"
              />
            </div>

            <p v-if="error" class="mt-3 text-[12px] text-danger" role="alert">{{ error }}</p>

            <div class="mt-4 flex items-center justify-end gap-2">
              <AppButton variant="ghost" size="sm" @click="closeForm">Cancel</AppButton>
              <AppButton variant="primary" size="sm" :loading="submitting" @click="save">
                {{ isNew ? 'Create' : 'Save' }}
              </AppButton>
            </div>
          </div>
        </div>
      </div>
    </div>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        :title="`Delete ${deleting.name}?`"
        body="Every task wearing this tag loses its badge. The tasks themselves are untouched."
        :blast="`${taskCount(deleting.id)} task${taskCount(deleting.id) === 1 ? '' : 's'} lose this badge`"
        confirm-label="Delete tag"
        @cancel="deleting = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </div>
</template>
