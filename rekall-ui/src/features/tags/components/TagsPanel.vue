<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import SkillMark from '@/features/skills/components/SkillMark.vue'
import TagSkillsField from '@/features/skills/components/TagSkillsField.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import TagIcon from '@/components/ui/TagIcon.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import AppDialog from '@/components/ui/AppDialog.vue'
import {
  TAG_COLOR_KEYS,
  TAG_COLOR_LABEL,
  TAG_ICON_KEYS,
  TAG_ICON_LABEL,
  type TagColorKey,
  type TagIconKey
} from '@/features/tags/model/tag'
import { skillsOfTag } from '@/features/skills/model/skill-assignment'
import type { Tag } from '@/features/catalog/model/catalog'
import type { TagId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const toast = useToastStore()
const closeButton = ref<HTMLButtonElement | null>(null)
const nameField = ref<HTMLInputElement | null>(null)

const editingId = ref<TagId | null>(null)
const isNew = ref(false)
const name = ref('')
const icon = ref<TagIconKey>('star')
const color = ref<TagColorKey>('crimson')
const skills = ref<readonly string[]>([])
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
  skills.value = []
  error.value = null
  focusName()
}

function openEdit(tag: Tag): void {
  isNew.value = false
  editingId.value = tag.id
  name.value = tag.name
  icon.value = TAG_ICON_KEYS.includes(tag.icon as TagIconKey) ? (tag.icon as TagIconKey) : 'star'
  color.value = TAG_COLOR_KEYS.includes(tag.color as TagColorKey) ? (tag.color as TagColorKey) : 'crimson'
  skills.value = skillsOfTag(tag.id, store.tagSkills).map((skill) => skill.name)
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
    let tagId = editingId.value
    if (tagId) await store.updateTag(tagId, input)
    else tagId = (await store.createTag(input)).id
    await store.assignTagSkills(tagId, skills.value)
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

function onDismiss(reason: 'escape' | 'backdrop'): void {
  if (reason === 'escape' && formOpen.value) closeForm()
  else emit('close')
}

onMounted(async () => {
  await store.refreshTags()
  await nextTick()
  closeButton.value?.focus()
})
</script>

<template>
  <AppDialog
    label="Tags"
    panel-class="flex flex-col max-h-[85vh] max-w-[560px]"
    panel-testid="tags-panel"
    @dismiss="onDismiss"
  >
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <span class="text-title font-semibold tracking-[-0.01em] text-text">Tags</span>
      <button
        ref="closeButton"
        type="button"
        class="focus-ring ml-auto grid size-7 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto px-6 py-5">
      <p class="mb-3 eyebrow text-caption">
        Configured tags
      </p>
      <p class="mb-4 text-small leading-relaxed text-text-subtle">
        A tag is a name, a glowing icon and a glow colour. Assign one to a task from the task's
        edit dialog; it shows as a badge wherever that task is listed.
      </p>

      <ul v-if="store.tags.length" class="space-y-2">
        <li
          v-for="tag in store.tags"
          :key="tag.id"
          class="flex items-center gap-3 rounded-control border border-border bg-canvas px-3.5 py-3"
          data-testid="tag-row"
        >
          <TagBadge size="md" :name="tag.name" :icon="tag.icon" :color="tag.color" />
          <SkillMark :skills="skillsOfTag(tag.id, store.tagSkills)" class="ml-1" />
          <span class="ml-auto shrink-0 text-caption text-text-subtle">
            {{ taskCount(tag.id) }} task{{ taskCount(tag.id) === 1 ? '' : 's' }}
          </span>
          <AppButton size="sm" variant="ghost" @click="openEdit(tag)">Edit</AppButton>
          <AppButton size="sm" variant="danger-quiet" @click="deleting = tag">Delete</AppButton>
        </li>
      </ul>
      <p v-else class="text-small text-text-subtle">
        No tags yet. Add the first one below.
      </p>

      <div class="mt-4">
        <AppButton v-if="!formOpen" variant="ghost" size="sm" @click="openCreate">
          + Add a tag
        </AppButton>

        <div
          v-else
          class="rounded-control border border-border-strong bg-canvas p-4"
          data-testid="tag-form"
        >
          <label for="tag-name" class="mb-1.5 block eyebrow text-caption">Name</label>
          <input
            id="tag-name"
            ref="nameField"
            v-model="name"
            data-testid="tag-name"
            class="field text-text h-9 w-full rounded-control px-3 text-body"
            placeholder="Bug, Idea, Blocked…"
            @keydown.enter="save"
          />

          <p class="mb-1.5 mt-4 eyebrow text-caption">Icon</p>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="option in TAG_ICON_KEYS"
              :key="option"
              type="button"
              class="focus-ring grid size-9 place-items-center rounded-key"
              :class="icon === option ? 'key-lit' : 'key-quiet'"
              :aria-pressed="icon === option"
              :aria-label="TAG_ICON_LABEL[option]"
              :title="TAG_ICON_LABEL[option]"
              @click="icon = option"
            >
              <TagIcon :icon="option" :color="color" :size="16" />
            </button>
          </div>

          <p class="mb-1.5 mt-4 eyebrow text-caption">Glow colour</p>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="option in TAG_COLOR_KEYS"
              :key="option"
              type="button"
              class="tag-bead focus-ring grid size-9 place-items-center rounded-full border-2 transition-colors"
              :class="color === option ? 'border-text' : 'border-transparent hover:border-border-strong'"
              :style="{ '--tag-tint': `var(--color-tag-${option})` }"
              :aria-pressed="color === option"
              :aria-label="TAG_COLOR_LABEL[option]"
              :title="TAG_COLOR_LABEL[option]"
              @click="color = option"
            />
          </div>

          <TagSkillsField v-model="skills" />

          <p v-if="error" class="mt-3 text-small text-danger" role="alert">{{ error }}</p>

          <div class="mt-4 flex items-center justify-end gap-2">
            <AppButton variant="ghost" size="sm" @click="closeForm">Cancel</AppButton>
            <AppButton variant="primary" size="sm" :loading="submitting" @click="save">
              {{ isNew ? 'Create' : 'Save' }}
            </AppButton>
          </div>
        </div>
      </div>
    </div>
    <template #overlay>
      <Transition name="dialog">
        <AppConfirm
          v-if="deleting"
          :title="`Delete ${deleting.name}?`"
          body="Every task wearing this tag loses its badge and the skills the tag brought. The tasks themselves are untouched."
          :blast="`${taskCount(deleting.id)} task${taskCount(deleting.id) === 1 ? '' : 's'} lose this badge`"
          confirm-label="Delete tag"
          @cancel="deleting = null"
          @confirm="confirmDelete"
        />
      </Transition>
    </template>
  </AppDialog>
</template>
