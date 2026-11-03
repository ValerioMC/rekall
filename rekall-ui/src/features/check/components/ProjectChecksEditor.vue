<script setup lang="ts">
import { computed, watch } from 'vue'
import CheckList from '@/features/check/components/CheckList.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useCheckStream } from '@/features/check/composables/useCheckStream'
import { useChecksStore } from '@/features/check/store/checks.store'
import type { Project } from '@/features/catalog/model/catalog'

/**
 * The commands every step of the project has to pass before a claim is accepted. A project with
 * none behaves as it always did.
 */
const props = defineProps<{ project: Project }>()

const store = useChecksStore()
const { run } = useAsyncAction()

const checks = computed(() => store.projectChecksOf(props.project.id))
const needsFolder = computed(() => checks.value.length > 0 && !props.project.repoFolder)

watch(
  () => props.project.id,
  (id) => void run(() => store.load(id)),
  { immediate: true }
)

useCheckStream({ onChecks: (event) => void store.applyProjectEvent(event.projectId) })
</script>

<template>
  <section aria-labelledby="project-checks-heading" data-testid="settings-checks">
    <h3 id="project-checks-heading" class="text-body font-semibold text-text">Done when</h3>
    <p class="mb-2 mt-0.5 text-small text-text-subtle">
      These commands run before a step's claim is accepted. If one fails, the claim is refused and the session
      sees its output.
    </p>
    <p v-if="needsFolder" class="mb-2 text-small text-warn" role="status" data-testid="checks-need-folder">
      Set the project's folder above: checks run inside it, and a claim is refused without one.
    </p>
    <CheckList
      :project-id="project.id"
      :step-id="null"
      :checks="checks"
      empty-text="No checks. A claim is accepted as soon as the session makes it."
    />
  </section>
</template>
