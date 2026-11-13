<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import CompanyRecordFields from '@/features/catalog/components/CompanyRecordFields.vue'
import ProjectRecordFields from '@/features/catalog/components/ProjectRecordFields.vue'
import TaskRecordFields from '@/features/catalog/components/TaskRecordFields.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useRecordWrites } from '@/composables/useRecordWrites'
import { identityHue } from '@/common/identity'
import { slugify } from '@/features/catalog/model/catalog'
import { labelOf, titleOf, type RecordDraft } from '@/features/catalog/model/record-draft'

const props = defineProps<{ draft: RecordDraft }>()
const emit = defineEmits<{ close: []; saved: [] }>()

const store = useConsoleStore()
const { run, isRunning } = useAsyncAction()

const form = ref<RecordDraft>({ ...props.draft })
const companyFields = ref<InstanceType<typeof CompanyRecordFields> | null>(null)
const projectFields = ref<InstanceType<typeof ProjectRecordFields> | null>(null)
const isConfirmingDelete = ref(false)
const submitted = ref(false)
const isPicking = ref(false)

const kind = props.draft.kind
const isNew = props.draft.id === null
const hasLabel = kind === 'project'

const KIND_NOUN = { company: 'company', project: 'project', task: 'task' } as const
const noun = props.draft.kind === 'task' && props.draft.taskKind === 'EPIC' ? 'epic' : KIND_NOUN[kind]
const heading = `${isNew ? 'New' : 'Edit'} ${noun}`

const title = computed(() => titleOf(form.value))
const label = computed(() => slugify(labelOf(form.value)))

/** The project a task lands in: its anchor opens the task's own. */
const parentProject = computed(() => {
  const current = form.value
  if (current.kind !== 'task') return undefined
  return store.projects.find((candidate) => candidate.id === current.projectId)
})

const titleError = computed(() => (submitted.value && !title.value.trim() ? 'A title is required.' : null))

const labelError = computed(() =>
  submitted.value && hasLabel && !label.value
    ? 'A label is required, and needs a letter or a digit in it.'
    : null
)

const canSave = computed(() => title.value.trim().length > 0 && (!hasLabel || label.value.length > 0))

const writes = useRecordWrites()
const blast = computed(() => writes.deletionBlast(form.value))

async function save(): Promise<void> {
  if (isRunning.value) return
  submitted.value = true
  if (!canSave.value) return
  const saved = await run(async () => {
    await writes.write(form.value, label.value, {
      company: async (companyId) => await companyFields.value?.applyImage(companyId),
      project: async (projectId) => await projectFields.value?.applyImage(projectId)
    })
    return true
  }, `${isNew ? 'Created' : 'Saved'} ${title.value.trim()}`)
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
  }, `Deleted ${title.value.trim()}`)
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

onMounted(() => window.addEventListener('keydown', saveOnModEnter, true))

onUnmounted(() => window.removeEventListener('keydown', saveOnModEnter, true))
</script>

<template>
  <AppDialog
    :label="heading"
    panel-class="max-w-[560px]"
    panel-testid="record-dialog"
    :dismissible="!isPicking"
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
        <span v-if="parentProject" class="flex items-center gap-1.5 truncate font-mono text-caption text-anchor/80">
          <span class="size-1.5 shrink-0 rounded-full" :style="{ backgroundColor: identityHue(parentProject.id).base }" aria-hidden="true" />
          <span class="min-w-0 truncate">{{ parentProject.anchor }}</span>
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
      <CompanyRecordFields
        v-if="form.kind === 'company'"
        ref="companyFields"
        v-model="form"
        :title-error="titleError"
      />
      <ProjectRecordFields
        v-else-if="form.kind === 'project'"
        ref="projectFields"
        v-model="form"
        :title-error="titleError"
        :label-error="labelError"
      />
      <TaskRecordFields
        v-else
        v-model="form"
        :title-error="titleError"
        @picking="isPicking = $event"
      />
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
          :title="`Delete ${title}?`"
          :body="
            kind === 'task'
              ? 'The notes on it are unlinked. Every one of them stays in the project, company or global list that owns it.'
              : 'Everything underneath goes with it. This is not recoverable.'
          "
          :blast="blast"
          :confirm-label="`Delete ${noun}`"
          @cancel="isConfirmingDelete = false"
          @confirm="remove"
        />
      </Transition>
    </template>
  </AppDialog>
</template>
