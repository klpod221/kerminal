<template>
  <Teleport to="body">
    <!-- Overlay -->
    <Transition
      enter-active-class="transition-opacity duration-300 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isVisible"
        class="fixed top-[30px] left-0 right-0 bottom-0 z-40 bg-black/50 backdrop-blur"
        @click="handleBackdropClick"
      ></div>
    </Transition>

    <!-- Modal Content -->
    <Transition
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0 scale-95 translate-y-4"
      enter-to-class="opacity-100 scale-100 translate-y-0"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100 scale-100 translate-y-0"
      leave-to-class="opacity-0 scale-95 translate-y-4"
    >
      <div
        v-if="isVisible"
        class="fixed top-[30px] left-0 right-0 bottom-0 z-50 flex items-center justify-center pointer-events-none"
      >
        <div
          class="relative bg-[#1a1a1a] border border-gray-700 rounded-lg shadow-2xl w-full mx-4 max-h-[90vh] overflow-hidden pointer-events-auto"
          :class="sizeClass"
          @click.stop
        >
          <!-- Header -->
          <div
            v-if="title || $slots.header || showCloseButton"
            class="flex items-center justify-between p-4 border-b border-gray-700"
          >
            <div class="flex items-center space-x-3">
              <div
                v-if="icon"
                class="rounded-lg p-2"
                :class="iconBackground || 'bg-blue-500/20'"
              >
                <component
                  :is="icon"
                  class="w-6 h-6"
                  :class="iconColor || 'text-blue-400'"
                />
              </div>
              <div>
                <h3 v-if="title" class="text-lg font-semibold text-white">
                  {{ title }}
                </h3>
                <slot name="header" />
              </div>
            </div>
            <Button
              v-if="showCloseButton"
              title="Close modal"
              variant="ghost"
              size="sm"
              :icon="X"
              @click="handleClose"
            />
          </div>

          <!-- Content -->
          <div class="p-4 overflow-y-auto max-h-[60vh]">
            <slot />
          </div>

          <!-- Footer -->
          <div
            v-if="$slots.footer"
            class="flex justify-end space-x-3 p-4 border-t border-gray-700 bg-[#171717]"
          >
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, watch, onMounted, onUnmounted } from "vue";
import { X } from "lucide-vue-next";
import Button from "./Button.vue";
import type { Component } from "vue";
import { useOverlay } from "../../composables/useOverlay";

interface ModalProps {
  id: string;
  visible?: boolean;
  title?: string;
  icon?: Component;
  iconBackground?: string;
  iconColor?: string;
  showCloseButton?: boolean;
  closeOnBackdrop?: boolean;
  closeOnEsc?: boolean;
  size?: "sm" | "md" | "lg" | "xl" | "2xl";
  parentId?: string;
  onBeforeOpen?: () => void | Promise<void>;
  onOpened?: () => void;
  onBeforeClose?: () => boolean | Promise<boolean>;
  onClosed?: () => void;
  onError?: (error: Error) => void;
}

const props = withDefaults(defineProps<ModalProps>(), {
  visible: false,
  showCloseButton: true,
  closeOnBackdrop: true,
  closeOnEsc: true,
  size: "md",
});

const emit = defineEmits(["close", "update:visible"]);

const {
  overlayStore,
  registerOverlay,
  unregisterOverlay,
  closeOverlay,
  isOverlayTransitioning,
} = useOverlay();

/**
 * Compute size class based on size prop
 */
const sizeClass = computed(() => {
  const sizeClasses = {
    sm: "max-w-sm",
    md: "max-w-md",
    lg: "max-w-lg",
    xl: "max-w-xl",
    "2xl": "max-w-2xl",
  };
  return sizeClasses[props.size];
});

const isVisible = computed(() => overlayStore.isVisible(props.id));
const isTransitioning = computed(() => isOverlayTransitioning(props.id));

/**
 * Handle close button click
 */
async function handleClose(): Promise<void> {
  if (isTransitioning.value) {
    console.warn(`⚠️ Modal ${props.id} is transitioning, ignoring close`);
    return;
  }

  await closeOverlay(props.id);
  emit("close");
  emit("update:visible", false);
}

/**
 * Handle backdrop click to close modal
 */
function handleBackdropClick(): void {
  if (props.closeOnBackdrop && !isTransitioning.value) {
    handleClose();
  }
}

/**
 * Handle keyboard events - close modal on Esc key
 */
function handleKeydown(event: KeyboardEvent): void {
  if (
    event.key === "Escape" &&
    props.closeOnEsc &&
    isVisible.value &&
    !isTransitioning.value
  ) {
    handleClose();
  }
}

onMounted(() => {
  registerOverlay({
    id: props.id,
    type: "modal",
    parentId: props.parentId || null,
    title: props.title,
    icon: props.icon,
    props: {
      size: props.size,
      iconBackground: props.iconBackground,
      iconColor: props.iconColor,
      showCloseButton: props.showCloseButton,
      closeOnBackdrop: props.closeOnBackdrop,
    },
    onBeforeOpen: props.onBeforeOpen,
    onOpened: props.onOpened,
    onBeforeClose: props.onBeforeClose,
    onClosed: props.onClosed,
    onError: props.onError,
  });
});

onUnmounted(() => {
  unregisterOverlay(props.id);
});

watch(
  () => props.visible,
  (newVisible) => {
    if (newVisible && !isVisible.value) {
      overlayStore.open(props.id);
    } else if (!newVisible && isVisible.value) {
      closeOverlay(props.id);
    }
  },
);

watch(isVisible, (visible) => {
  if (visible) {
    document.body.style.overflow = "hidden";
    document.addEventListener("keydown", handleKeydown);
  } else {
    document.body.style.overflow = "";
    document.removeEventListener("keydown", handleKeydown);
  }
});

onUnmounted(() => {
  document.body.style.overflow = "";
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-content-enter-active,
.modal-content-leave-active {
  transition: all 0.3s ease;
}

.modal-content-enter-from,
.modal-content-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(1rem);
}
</style>
