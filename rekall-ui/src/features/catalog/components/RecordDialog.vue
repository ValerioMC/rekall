<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import ProjectPicker from '@/features/catalog/components/ProjectPicker.vue'
import CompanyLogoField from '@/features/catalog/components/record/CompanyLogoField.vue'
import ProjectIconField from '@/features/catalog/components/record/ProjectIconField.vue'
import RecordIdentityFields from '@/features/catalog/components/record/RecordIdentityFields.vue'
import TaskSkillsField from '@/features/catalog/components/record/TaskSkillsField.vue'
import TaskTagField from '@/features/catalog/components/record/TaskTagField.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useRecordWrites } from '@/composables/useRecordWrites'
import AppDialog from '@/components/ui/AppDialog.vue'
import { identityHue } from '@/common/identity'
import {
  PROJECT_STATUSES,
  PROJECT_STATUS_LABEL,
  TASK_STATUSES,
  TASK_STATUS_LABEL,
  slugify
} from '@/features/catalog/model/catalog'
import type { ProjectStatus, TaskStatus } from '@/features/catalog/model/catalog'
import type { RecordDraft } from '@/features/catalog/model/record-draft'
import type { TagId } from '@/model/branded'

const props = defineProps<{ draft: RecordDraft }>()
const emit = defineEmits<{ close: []; saved: [] }>()

const store = useConsoleStore()
const { run, isRunning } = useAsyncAction()

const form = ref<RecordDraft>({ ...props.draft })
const identity = ref<InstanceType<typeof RecordIdentityFields> | null>(null)
const iconField = ref<InstanceType<typeof ProjectIconField> | null>(null)
const logoField = ref<InstanceType<typeof CompanyLogoField> | null>(null)
const isConfirmingDelete = ref(false)
const submitted = ref(false)
const isPickingProject = ref(false)

const kind = props.draft.kind
const isNew = props.draft.id === null
const hasLabel = kind !== 'company'
const originalLabel = props.draft.kind === 'company' ? '' : props.draft.label

// A task's description is written in its own pane, never squeezed in here.
const asksForDescription = kind !== 'task'

const KIND_NOUN = { company: 'company', project: 'project', task: 'task' } as const
const heading = `${isNew ? 'New' : 'Edit'} ${KIND_NOUN[kind]}`

const titleValue = computed<string>({
  get: () => (form.value.kind === 'company' ? form.value.name : form.value.title),
  set: (value) => {
    if (form.value.kind === 'company') form.value.name = value
    else form.value.title = value
  }
})

const rawLabel = computed<string>({
  get: () => (form.value.kind === 'company' ? '' : form.value.label),
  set: (value) => {
    if (form.value.kind !== 'company') form.value.label = value
  }
})

const description = computed<string>({
  get: () => form.value.description,
  set: (value) => {
    form.value.description = value
  }
})

const status = computed<string>({
  get: () => (form.value.kind === 'company' ? '' : form.value.status),
  set: (value) => {
    if (form.value.kind === 'project') form.value.status = value as ProjectStatus
    else if (form.value.kind === 'task') form.value.status = value as TaskStatus
  }
})

const label = computed(() => slugify(rawLabel.value))

/** The parent's anchor a task's own is appended to: its project's. */
const anchorParent = computed(() => {
  const current = form.value
  if (current.kind !== 'task') return ''
  return store.projects.find((candidate) => candidate.id === current.projectId)?.anchor ?? ''
})

const parentId = computed<string>({
  get: () => {
    const current = form.value
    if (current.kind === 'project') return current.companyId
    if (current.kind === 'task') return current.projectId
    return ''
  },
  set: (value) => {
    const current = form.value
    if (current.kind === 'project') current.companyId = value as typeof current.companyId
    else if (current.kind === 'task') current.projectId = value as typeof current.projectId
  }
})

const companyOptions = computed(() =>
  kind === 'project'
    ? store.companies.map((company) => ({
        value: company.id as string,
        label: `${company.name}  ·  company:${company.name}`
      }))
    : []
)

const parentLabel = kind === 'task' ? 'Project' : 'Company'

const statusOptions =
  kind === 'task'
    ? TASK_STATUSES.map((value) => ({ value, label: TASK_STATUS_LABEL[value] }))
    : PROJECT_STATUSES.map((value) => ({ value, label: PROJECT_STATUS_LABEL[value] }))

const tagId = computed<TagId | null>({
  get: () => (form.value.kind === 'task' ? form.value.tagId : null),
  set: (value) => {
    if (form.value.kind === 'task') form.value.tagId = value
  }
})

const skills = computed<readonly string[]>({
  get: () => (form.value.kind === 'task' ? form.value.skills : []),
  set: (value) => {
    if (form.value.kind === 'task') form.value.skills = [...value]
  }
})

const projectIcon = computed<string>({
  get: () => (form.value.kind === 'project' ? form.value.icon : 'folder'),
  set: (value) => {
    if (form.value.kind === 'project') form.value.icon = value
  }
})

const titleError = computed(() =>
  submitted.value && !titleValue.value.trim() ? 'A title is required.' : null
)

const labelError = computed(() =>
  submitted.value && hasLabel && !label.value
    ? 'A label is required, and needs a letter or a digit in it.'
    : null
)

const canSave = computed(
  () => titleValue.value.trim().length > 0 && (!hasLabel || label.value.length > 0)
)

const writes = useRecordWrites()
const blast = computed(() => writes.deletionBlast(form.value))

async function save(): Promise<void> {
  if (isRunning.value) return
  submitted.value = true
  if (!canSave.value) return
  const saved = await run(async () => {
    await writes.write(form.value, label.value, {
      company: async (companyId) => await logoField.value?.apply(companyId),
      project: async (projectId) => await iconField.value?.apply(projectId)
    })
    return true
  }, `${isNew ? 'Created' : 'Saved'} ${titleValue.value.trim()}`)
  if (saved) {
    emit('saved')
    emit('close')
  }
}

async function remove(): Promise<void> {
  if (form.value.id === null) return
  const done = await run(async () => {
    await writes.erase(form.value)
    return true
  }, `Deleted ${titleValue.value.trim()}`)
  isConfirmingDelete.value = false
  if (done) {
    emit('saved')
    emit('close')
  }
}

function saveOnModEnter(event: KeyboardEvent): void {
  if (!(event.metaKey || event.ctrlKey) || event.key !== 'Enter') return
  event.preventDefault()
  event.stopPropagation()
  void save()
}

onMounted(async () => {
  window.addEventListener('keydown', saveOnModEnter, true)
  await nextTick()
  identity.value?.focusTitle()
})

onUnmounted(() => window.removeEventListener('keydown', saveOnModEnter, true))
</script>

<template>
  <AppDialog
    :label="heading"
    panel-class="max-w-[560px]"
    panel-testid="record-dialog"
    :dismissible="!isPickingProject"
    @dismiss="emit('close')"
  >
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <span
        class="grid size-8 shrink-0 place-items-center rounded-card border border-border-strong font-mono text-body font-semibold text-accent"
        style="background-image: linear-gradient(155deg, var(--color-surface-raised), var(--color-canvas))"
        aria-hidden="true"
      >
        {{ kind.charAt(0).toUpperCase() }}
      </span>
      <span class="min-w-0 flex-1">
        <span class="block text-title font-semibold tracking-[-0.01em] text-text">
          {{ heading }}
        </span>
        <span v-if="anchorParent" class="flex items-center gap-1.5 truncate font-mono text-caption text-anchor/80">
          <span v-if="kind === 'task'" class="size-1.5 shrink-0 rounded-full" :style="{ backgroundColor: identityHue(parentId).base }" aria-hidden="true" />
          <span class="min-w-0 truncate">{{ anchorParent }}</span>
        </span>
      </span>
      <button
        class="focus-ring grid size-7 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>

    <div class="max-h-[62vh] overflow-y-auto px-6 py-5">
      <template v-if="kind === 'task' || companyOptions.length">
        <label
          for="record-parent"
          class="mb-1.5 block eyebrow text-caption"
        >
          {{ parentLabel }}
        </label>
        <ProjectPicker
          v-if="kind === 'task'"
          id="record-parent"
          v-model="parentId"
          data-testid="record-parent"
          :projects="store.projects"
          @open-change="isPickingProject = $event"
        />
        <AppSelect
          v-else
          id="record-parent"
          v-model="parentId"
          data-testid="record-parent"
          :options="companyOptions"
        />
        <p class="mb-5 mt-1.5 text-caption text-text-subtle">
          <template v-if="isNew">Where this lands. It opens the anchor.</template>
          <template v-else>Changing this moves the record, and its anchor with it.</template>
        </p>
      </template>

      <RecordIdentityFields
        ref="identity"
        v-model:title="titleValue"
        v-model:label="rawLabel"
        :kind="kind"
        :is-new="isNew"
        :original-label="originalLabel"
        :anchor-parent="anchorParent"
        :parent-id="parentId"
        :title-error="titleError"
        :label-error="labelError"
      />

      <template v-if="hasLabel">
        <p class="mb-1.5 mt-5 eyebrow text-caption">
          Status
        </p>
        <div class="flex flex-wrap gap-1.5">
          <button
            v-for="option in statusOptions"
            :key="option.value"
            type="button"
            class="focus-ring h-8 rounded-control border px-3 text-small transition-colors"
            :class="
              status === option.value
                ? 'border-accent bg-accent-soft text-accent'
                : 'border-border-strong bg-canvas text-text-muted hover:border-text-subtle hover:text-text'
            "
            :aria-pressed="status === option.value"
            @click="status = option.value"
          >
            {{ option.label }}
          </button>
        </div>
      </template>

      <ProjectIconField v-if="form.kind === 'project'" ref="iconField" v-model="projectIcon" :project-id="form.id" />

      <CompanyLogoField v-if="form.kind === 'company'" ref="logoField" :company-id="form.id" :name="titleValue" />

      <TaskTagField v-if="kind === 'task'" v-model="tagId" />

      <TaskSkillsField v-if="kind === 'task' && form.id === null" v-model="skills" :tag-id="tagId" />

      <template v-if="asksForDescription">
        <label
          for="record-description"
          class="mb-1.5 mt-5 block eyebrow text-caption"
        >
          Description
        </label>
        <textarea
          id="record-description"
          v-model="description"
          data-testid="record-description"
          rows="3"
          class="field text-text w-full resize-y rounded-control p-3 text-body leading-relaxed"
          placeholder="What it is, in a sentence. This travels with the record into every context."
        />
      </template>
    </div>

    <footer class="flex items-center gap-2 border-t border-border bg-canvas px-6 py-3.5">
      <AppButton
        v-if="!isNew"
        variant="danger"
        size="sm"
        data-testid="record-delete"
        @click="isConfirmingDelete = true"
      >
        Delete
      </AppButton>
      <span class="ml-auto flex items-center gap-2">
        <kbd
          class="hidden rounded-key border border-border px-1.5 py-0.5 font-mono text-micro text-text-subtle sm:inline"
        >
          &#8984;&crarr;
        </kbd>
        <AppButton variant="ghost" size="sm" @click="emit('close')">Cancel</AppButton>
        <AppButton
          variant="primary"
          size="sm"
          data-testid="record-save"
          :disabled="!canSave || isRunning"
          @click="save"
        >
          {{ isNew ? 'Create' : 'Save' }}
        </AppButton>
      </span>
    </footer>
    <template #overlay>
      <Transition name="dialog">
        <AppConfirm
          v-if="isConfirmingDelete"
          :title="`Delete ${titleValue}?`"
          :body="
            kind === 'task'
              ? 'The notes on it are unlinked. Every one of them stays in the project, company or global list that owns it.'
              : 'Everything underneath goes with it. This is not recoverable.'
          "
          :blast="blast"
          :confirm-label="`Delete ${KIND_NOUN[kind]}`"
          @cancel="isConfirmingDelete = false"
          @confirm="remove"
        />
      </Transition>
    </template>
  </AppDialog>
</template>
