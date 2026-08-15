<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import {
  PROJECT_STATUSES,
  PROJECT_STATUS_LABEL,
  TASK_STATUSES,
  TASK_STATUS_LABEL,
  slugify
} from '@/model/catalog'
import type { ProjectStatus, TaskStatus } from '@/model/catalog'
import type { RecordDraft } from '@/model/record-draft'

/**
 * One editor for all three levels, creating and editing alike.
 *
 * A form per level and per direction is four screens that drift apart, and the thing that
 * matters here is the same on every one of them: what the record is called, and what has to be
 * typed to load it. Those two are shown together, always, with the anchor assembled live
 * underneath. Nobody should have to save a record to find out what its anchor became.
 */
const props = defineProps<{ draft: RecordDraft }>()
const emit = defineEmits<{ close: []; saved: [] }>()

const store = useConsoleStore()
const { run, isRunning } = useAsyncAction()

const form = ref<RecordDraft>({ ...props.draft })
const firstField = ref<HTMLInputElement | null>(null)
const isConfirmingDelete = ref(false)
const submitted = ref(false)

const kind = props.draft.kind
const isNew = props.draft.id === null
const hasLabel = kind !== 'company'
const originalLabel = props.draft.kind === 'company' ? '' : props.draft.label

const KIND_NOUN = { company: 'company', project: 'project', task: 'task' } as const
const heading = `${isNew ? 'New' : 'Edit'} ${KIND_NOUN[kind]}`

/*
 * The union is unpacked into flat accessors here rather than narrowed in the markup. A
 * template that has to prove which variant it is holding before it can bind a field is a
 * template nobody wants to change later.
 */
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

/**
 * The label follows the title until the label is touched, and never afterwards.
 *
 * Typing a title and getting `code-validator` for free is most of the value; having it
 * overwrite a label you deliberately chose would be the opposite.
 */
const labelWasTouched = ref(!isNew)

function onTitleInput(value: string): void {
  titleValue.value = value
  if (hasLabel && !labelWasTouched.value) rawLabel.value = slugify(value)
}

/**
 * The raw text is kept while typing and narrowed for the anchor and the save, so `code-` on
 * the way to `code-validator` is not swallowed one keystroke at a time.
 */
function onLabelInput(value: string): void {
  labelWasTouched.value = true
  rawLabel.value = value
}

const label = computed(() => slugify(rawLabel.value))

/**
 * The part of the anchor the parent contributes, which is settled before you type anything.
 *
 * Shown apart from the rest because it is not in play: a task's anchor opens with the project
 * it belongs to, and that is a fact of where you are, not a field you are filling in.
 */
const anchorParent = computed(() => {
  const current = form.value
  if (current.kind !== 'task') return ''
  const project = store.projects.find((candidate) => candidate.id === current.projectId)
  return project ? project.anchor : ''
})

/** The part this record contributes, which is what the label field is deciding. */
const anchorSelf = computed(() => {
  const current = form.value
  if (current.kind === 'company') {
    return current.name.trim() ? `company:${current.name.trim()}` : ''
  }
  if (!label.value) return ''
  return `${current.kind}:${label.value}`
})

/** The whole anchor, as `/rk` would take it. */
const anchorPreview = computed(() =>
  [anchorParent.value, anchorSelf.value].filter(Boolean).join(' ')
)

/**
 * The parent, as a field rather than as something the screen decided on your behalf.
 *
 * It used to be inferred from the scope, which is right when you are inside a project and a
 * silent guess when you are not: a task would land in whichever project happened to sort first
 * and nothing said so. It is also the only way to move a record afterwards, which the endpoint
 * has always supported.
 */
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

/** Each option carries the anchor it contributes, so the grammar is legible in the list too. */
const parentOptions = computed(() => {
  if (kind === 'project') {
    return store.companies.map((company) => ({
      value: company.id as string,
      label: `${company.name}  ·  company:${company.name}`
    }))
  }
  if (kind === 'task') {
    return store.projects.map((project) => ({
      value: project.id as string,
      label: `${project.companyName} / ${project.title}  ·  ${project.anchor}`
    }))
  }
  return []
})

const parentLabel = kind === 'task' ? 'Project' : 'Company'

/** Renaming the label moves the anchor, and what breaks is outside this application. */
const anchorIsMoving = computed(() => !isNew && hasLabel && label.value !== originalLabel)

const statusOptions =
  kind === 'task'
    ? TASK_STATUSES.map((value) => ({ value, label: TASK_STATUS_LABEL[value] }))
    : PROJECT_STATUSES.map((value) => ({ value, label: PROJECT_STATUS_LABEL[value] }))

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

/** Counts come from what is loaded, so the warning is the real blast radius and not a guess. */
const blast = computed(() => {
  const current = form.value
  if (current.kind === 'company') {
    const company = store.companies.find((candidate) => candidate.id === current.id)
    return `${plural(company?.projectCount ?? 0, 'project')} · ${plural(company?.taskCount ?? 0, 'task')} · every note left on nothing`
  }
  if (current.kind === 'project') {
    const count = store.tasks.filter((task) => task.projectId === current.id).length
    return `${plural(count, 'task')} · every note left on nothing`
  }
  const task = store.tasks.find((candidate) => candidate.id === current.id)
  return `${plural(task?.documentCount ?? 0, 'note')} unlinked · those left on no task are removed`
})

function plural(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? '' : 's'}`
}

async function save(): Promise<void> {
  submitted.value = true
  if (!canSave.value) return
  const current = form.value
  const trimmedDescription = current.description.trim() === '' ? null : current.description.trim()

  const saved = await run(async () => {
    if (current.kind === 'company') {
      const input = { name: current.name.trim(), description: trimmedDescription }
      if (current.id === null) await store.createCompany(input)
      else await store.updateCompany(current.id, input)
    } else if (current.kind === 'project') {
      const input = {
        label: label.value,
        title: current.title.trim(),
        status: current.status,
        description: trimmedDescription,
        companyId: current.companyId
      }
      if (current.id === null) await store.createProject(input)
      else await store.updateProject(current.id, input)
    } else {
      const input = {
        label: label.value,
        title: current.title.trim(),
        status: current.status,
        description: trimmedDescription,
        projectId: current.projectId
      }
      if (current.id === null) await store.createTask(input)
      else await store.updateTask(current.id, input)
    }
    return true
  }, `${isNew ? 'Created' : 'Saved'} ${titleValue.value.trim()}`)

  if (saved) {
    emit('saved')
    emit('close')
  }
}

async function remove(): Promise<void> {
  const current = form.value
  if (current.id === null) return

  const done = await run(async () => {
    if (current.kind === 'company' && current.id) await store.deleteCompany(current.id)
    else if (current.kind === 'project' && current.id) await store.deleteProject(current.id)
    else if (current.kind === 'task' && current.id) await store.deleteTask(current.id)
    return true
  }, `Deleted ${titleValue.value.trim()}`)

  isConfirmingDelete.value = false
  if (done) {
    emit('saved')
    emit('close')
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && !isConfirmingDelete.value) {
    event.stopPropagation()
    emit('close')
    return
  }
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault()
    event.stopPropagation()
    void save()
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown, true)
  await nextTick()
  firstField.value?.focus()
  firstField.value?.select()
})

onUnmounted(() => window.removeEventListener('keydown', onKeydown, true))
</script>

<template>
  <div
    class="fade-in fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <div
      class="rise w-full max-w-[560px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
      role="dialog"
      aria-modal="true"
      :aria-label="heading"
      data-testid="record-dialog"
    >
      <header class="flex items-center gap-3 border-b border-border px-6 py-4">
        <span
          class="grid size-8 shrink-0 place-items-center rounded-[9px] border border-border-strong bg-canvas font-mono text-[13px] text-accent"
          aria-hidden="true"
        >
          {{ kind.charAt(0).toUpperCase() }}
        </span>
        <span class="min-w-0 flex-1">
          <span class="block text-[15px] font-semibold tracking-[-0.01em] text-text">
            {{ heading }}
          </span>
          <span v-if="anchorParent" class="block truncate font-mono text-[11px] text-anchor/80">
            {{ anchorParent }}
          </span>
        </span>
        <button
          class="focus-ring grid size-7 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
          aria-label="Close"
          @click="emit('close')"
        >
          &times;
        </button>
      </header>

      <div class="max-h-[62vh] overflow-y-auto px-6 py-5">
        <!-- Where it lives, first, because it is the half of the anchor already settled. -->
        <template v-if="parentOptions.length">
          <label
            for="record-parent"
            class="mb-1.5 block text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle"
          >
            {{ parentLabel }}
          </label>
          <AppSelect
            id="record-parent"
            v-model="parentId"
            data-testid="record-parent"
            :options="parentOptions"
          />
          <p class="mb-5 mt-1.5 text-[11.5px] text-text-subtle">
            <template v-if="isNew">Where this lands. It opens the anchor.</template>
            <template v-else>Changing this moves the record, and its anchor with it.</template>
          </p>
        </template>

        <label
          for="record-title"
          class="mb-1.5 block text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle"
        >
          Title
        </label>
        <input
          id="record-title"
          ref="firstField"
          :value="titleValue"
          data-testid="record-title"
          class="focus-ring h-10 w-full rounded-[var(--radius-control)] border border-border bg-canvas px-3 text-[15px] text-text outline-none transition-colors placeholder:text-text-subtle hover:border-border-strong focus:border-accent"
          :class="titleError && 'border-danger'"
          :placeholder="hasLabel ? 'Code validator, main workflow' : 'Acme S.p.A.'"
          @input="onTitleInput(($event.target as HTMLInputElement).value)"
        />
        <p v-if="titleError" class="mt-1.5 text-[11.5px] text-danger" role="alert">
          {{ titleError }}
        </p>
        <p v-else class="mt-1.5 text-[11.5px] text-text-subtle">
          What this is called. Change it whenever a better name turns up.
        </p>

        <template v-if="hasLabel">
          <label
            for="record-label"
            class="mb-1.5 mt-5 block text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle"
          >
            Label
          </label>
          <input
            id="record-label"
            :value="rawLabel"
            data-testid="record-label"
            spellcheck="false"
            autocomplete="off"
            class="focus-ring h-10 w-full rounded-[var(--radius-control)] border border-border bg-canvas px-3 font-mono text-[14px] text-anchor outline-none transition-colors placeholder:text-text-subtle hover:border-border-strong focus:border-anchor"
            :class="labelError && 'border-danger'"
            placeholder="code-validator"
            @input="onLabelInput(($event.target as HTMLInputElement).value)"
          />
          <p v-if="labelError" class="mt-1.5 text-[11.5px] text-danger" role="alert">
            {{ labelError }}
          </p>
          <p v-else class="mt-1.5 text-[11.5px] text-text-subtle">
            What the anchor carries. Lowercase, no spaces, unique inside its parent.
          </p>
        </template>

        <!-- The anchor, assembled as it is typed. This is the product, so it is not a footnote. -->
        <div
          class="mt-4 flex items-center gap-3 rounded-[var(--radius-control)] border border-anchor-line bg-anchor-soft px-3.5 py-3"
          data-testid="anchor-preview"
          :title="anchorPreview"
        >
          <span class="shrink-0 font-mono text-[12px] text-text-subtle" aria-hidden="true">/rk</span>
          <span class="min-w-0 flex-1 truncate font-mono text-[13px]">
            <!-- The parent's half is settled and reads that way; the rest is what you are typing.
                 Both spans sit on one line so the separating space survives into the text. -->
            <span v-if="anchorParent" class="text-anchor/60">{{ anchorParent }}</span> <span class="text-anchor">{{ anchorSelf || '…' }}</span>
          </span>
          <span class="shrink-0 text-[10.5px] uppercase tracking-[0.09em] text-text-subtle">
            loads this
          </span>
        </div>

        <p
          v-if="anchorIsMoving"
          class="mt-2.5 rounded-[var(--radius-control)] border border-warn bg-warn-soft px-3.5 py-2.5 text-[12px] leading-relaxed text-warn"
          data-testid="rename-warning"
        >
          The anchor moves with this label. Anything still typing
          <code>{{ kind }}:{{ originalLabel }}</code> will stop finding it.
        </p>

        <template v-if="hasLabel">
          <p class="mb-1.5 mt-5 text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
            Status
          </p>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="option in statusOptions"
              :key="option.value"
              type="button"
              class="focus-ring h-8 rounded-[var(--radius-control)] border px-3 text-[12.5px] transition-colors"
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

        <label
          for="record-description"
          class="mb-1.5 mt-5 block text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle"
        >
          Description
        </label>
        <textarea
          id="record-description"
          v-model="description"
          data-testid="record-description"
          rows="3"
          class="focus-ring w-full resize-y rounded-[var(--radius-control)] border border-border bg-canvas p-3 text-[13px] leading-relaxed text-text outline-none transition-colors placeholder:text-text-subtle hover:border-border-strong focus:border-accent"
          placeholder="What it is, in a sentence. This travels with the record into every context."
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
            class="hidden rounded border border-border px-1.5 py-0.5 font-mono text-[10px] text-text-subtle sm:inline"
          >
            &#8984;&crarr;
          </kbd>
          <AppButton variant="ghost" size="sm" @click="emit('close')">Cancel</AppButton>
          <AppButton
            variant="primary"
            size="sm"
            data-testid="record-save"
            :loading="isRunning"
            :disabled="!canSave"
            @click="save"
          >
            {{ isNew ? 'Create' : 'Save' }}
          </AppButton>
        </span>
      </footer>
    </div>

    <AppConfirm
      v-if="isConfirmingDelete"
      :title="`Delete ${titleValue}?`"
      :body="
        kind === 'task'
          ? 'The notes on it are unlinked. A note that other tasks still use survives.'
          : 'Everything underneath goes with it. This is not recoverable.'
      "
      :blast="blast"
      :confirm-label="`Delete ${KIND_NOUN[kind]}`"
      @cancel="isConfirmingDelete = false"
      @confirm="remove"
    />
  </div>
</template>
