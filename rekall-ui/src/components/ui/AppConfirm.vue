<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'

defineProps<{
  title: string
  body: string
  blast: string
  confirmLabel: string
}>()

const emit = defineEmits<{ cancel: []; confirm: [] }>()

const cancelButton = ref<InstanceType<typeof AppButton> | null>(null)

onMounted(async () => {
  await nextTick()
  ;(cancelButton.value?.$el as HTMLElement | undefined)?.focus()
})
</script>

<template>
  <AppDialog :label="title" role="alertdialog" panel-class="max-w-[440px]" @dismiss="emit('cancel')">
    <div class="p-5">
      <h2 class="mb-2 text-title font-semibold text-text">{{ title }}</h2>
      <p class="text-body leading-relaxed text-text-muted">{{ body }}</p>
      <p
        class="relative mt-3.5 overflow-hidden rounded-control border border-danger bg-danger-soft py-2.5 pl-4 pr-3 font-mono text-small text-danger"
      >
        <span
          class="absolute inset-y-0 left-0 w-2.5"
          style="background-image: repeating-linear-gradient(135deg, var(--color-danger) 0 3px, transparent 3px 7px); opacity: 0.55"
          aria-hidden="true"
        />
        {{ blast }}
      </p>
    </div>
    <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
      <AppButton ref="cancelButton" variant="ghost" size="sm" @click="emit('cancel')">
        Keep it
      </AppButton>
      <AppButton variant="danger" size="sm" @click="emit('confirm')">
        {{ confirmLabel }}
      </AppButton>
    </div>
  </AppDialog>
</template>
