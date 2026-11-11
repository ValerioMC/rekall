<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import ProjectIconField from '@/features/catalog/components/record/ProjectIconField.vue'
import RecordIdentityFields from '@/features/catalog/components/record/RecordIdentityFields.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useRecordWrites } from '@/composables/useRecordWrites'
import { useConsoleStore } from '@/stores/console.store'
import { PROJECT_STATUSES, PROJECT_STATUS_LABEL, slugify } from '@/features/catalog/model/catalog'
import { projectDraft, type ProjectDraft } from '@/features/catalog/model/record-draft'
import type { Project } from '@/features/catalog/model/catalog'

/**
 * What the project is called and where it lives: title, label, company, status, icon and
 * description, saved together because a new label moves the anchor. Deleting it is the last group.
 */
const props = defineProps<{ project: Project }>()

const emit = defineEmits<{ deleted: [] }>()

const store = useConsoleStore()
const writes = useRecordWrites()
const { run, isRunning } = useAsyncAction()

const baseline = ref<ProjectDraft>(projectDraft(props.project.companyId, props.project))
const form = ref<ProjectDraft>({ ...baseline.value })
const iconField = ref<InstanceType<typeof ProjectIconField> | null>(null)
const iconKey = ref(0)
const submitted = ref(false)
const isConfirmingDelete = ref(false)

const label = computed(() => slugify(form.value.label))

const titleError = computed(() => (submitted.value && !form.value.title.trim() ? 'A title is required.' : null))
const labelError = computed(() =>
  submitted.value && !label.value ? 'A label is required, and needs a letter or a digit in it.' : null
)
const canSave = computed(() => form.value.title.trim().length > 0 && label.value.length > 0)

const isDirty = computed(() => {
  const saved = baseline.value
  const edited = form.value
  return (
    edited.title !== saved.title ||
    label.value !== saved.label ||
    edited.description !== saved.description ||
    edited.status !== saved.status ||
    edited.icon !== saved.icon ||
    edited.companyId !== saved.companyId ||
    (iconField.value?.hasPendingChange ?? false)
  )
})

const companyOptions = computed(() =>
  store.companies.map((company) => ({ value: company.id as string, label: company.name }))
)

const statusOptions = PROJECT_STATUSES.map((value) => ({ value, label: PROJECT_STATUS_LABEL[value] }))
const blast = computed(() => writes.deletionBlast(form.value))

function startFrom(project: Project): void {
  baseline.value = projectDraft(project.companyId, project)
  form.value = { ...baseline.value }
  iconKey.value += 1
  submitted.value = false
}

async function save(): Promise<void> {
  if (isRunning.value) return
  submitted.value = true
  if (!canSave.value) return
  const saved = await run(async () => {
    await writes.write(form.value, label.value, {
      company: async () => undefined,
      project: async (projectId) => await iconField.value?.apply(projectId)
    })
    return true
  }, `Saved ${form.value.title.trim()}`)
  if (!saved) return
  await nextTick()
  startFrom(props.project)
}

async function remove(): Promise<void> {
  const title = form.value.title.trim()
  const done = await run(async () => {
    await writes.erase(form.value)
    return true
  }, `Deleted ${title}`)
  isConfirmingDelete.value = false
  if (done) emit('deleted')
}

watch(() => props.project.id, () => startFrom(props.project))
</script>

<template>
  <section class="relative flex flex-col" aria-label="General" data-testid="settings-general">
    <div class="flex flex-col gap-7">
      <div>
        <RecordIdentityFields
          v-model:title="form.title"
          v-model:label="form.label"
          kind="project"
          :is-new="false"
          :original-label="baseline.label"
          anchor-parent=""
          parent-id=""
          :title-error="titleError"
          :label-error="labelError"
        />
      </div>

      <div>
        <label for="project-company" class="mb-1.5 block eyebrow text-caption">Company</label>
        <AppSelect id="project-company" v-model="form.companyId" data-testid="project-company" :options="companyOptions" />
        <p class="mt-1.5 text-caption text-text-subtle">Changing this moves the project, and its anchor with it.</p>
      </div>

      <div>
        <p class="mb-1.5 eyebrow text-caption">Status</p>
        <div class="flex flex-wrap gap-1.5" role="group" aria-label="Status">
          <button
            v-for="option in statusOptions"
            :key="option.value"
            type="button"
            class="focus-ring h-8 rounded-control border px-3 text-small transition-colors"
            :class="
              form.status === option.value
                ? 'border-accent bg-accent-soft text-accent'
                : 'border-border-strong bg-canvas text-text-muted hover:border-text-subtle hover:text-text'
            "
            :aria-pressed="form.status === option.value"
            :data-testid="`project-status-${option.value}`"
            @click="form.status = option.value"
          >
            {{ option.label }}
          </button>
        </div>
      </div>

      <div>
        <ProjectIconField :key="iconKey" ref="iconField" v-model="form.icon" :project-id="project.id" />
      </div>

      <div>
        <label for="project-description" class="mb-1.5 block eyebrow text-caption">Description</label>
        <textarea
          id="project-description"
          v-model="form.description"
          data-testid="project-description"
          rows="4"
          class="field text-text block w-full resize-y rounded-control p-3 text-body leading-relaxed"
          placeholder="What this is, in a few sentences."
        />
        <p class="mt-1.5 text-caption text-text-subtle">It travels into every context that loads the project.</p>
      </div>

      <div class="border-t border-border pt-5">
        <h3 class="text-body font-semibold text-text">Delete project</h3>
        <p class="mb-3 mt-0.5 text-small text-text-subtle">
          Its tasks and the notes it owns go with it. This is not recoverable.
        </p>
        <AppButton variant="danger" size="sm" data-testid="project-delete" @click="isConfirmingDelete = true">
          Delete project
        </AppButton>
      </div>
    </div>

    <Transition name="fade-quick">
      <div
        v-if="isDirty"
        class="sticky bottom-0 -mx-4 mt-6 flex items-center gap-2 border-t border-border bg-surface px-4 py-3"
        role="group"
        aria-label="Unsaved changes"
        data-testid="settings-save-bar"
      >
        <span class="mr-auto text-small text-warn">Unsaved changes</span>
        <AppButton variant="ghost" size="sm" data-testid="settings-revert" @click="startFrom(project)">Revert</AppButton>
        <AppButton variant="primary" size="sm" data-testid="settings-save" :disabled="isRunning" @click="save">Save</AppButton>
      </div>
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="isConfirmingDelete"
        :title="`Delete ${project.title}?`"
        body="Everything underneath goes with it. This is not recoverable."
        :blast="blast"
        confirm-label="Delete project"
        @cancel="isConfirmingDelete = false"
        @confirm="remove"
      />
    </Transition>
  </section>
</template>
