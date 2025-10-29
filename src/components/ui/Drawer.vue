<template>
  <Teleport to="body">
    <!-- Overlay -->
    <Transition
      enter-active-class="transition-opacity duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-300"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isVisible"
        class="fixed inset-0 bg-black/50 z-40 top-[30px] backdrop-blur"
        @click="handleOverlayClick"
      ></div>
    </Transition>

    <!-- Drawer -->
    <Transition
      :enter-active-class="`transition-transform duration-300 ease-out`"
      :enter-from-class="
        position === 'left' ? '-translate-x-full' : 'translate-x-full'
      "
      :enter-to-class="'translate-x-0'"
      :leave-active-class="`transition-transform duration-300 ease-in`"
      :leave-from-class="'translate-x-0'"
      :leave-to-class="
        position === 'left' ? '-translate-x-full' : 'translate-x-full'
      "
    >
      <div
        v-if="isVisible"
        class="no-drag fixed z-50 bg-[#1a1a1a] border-gray-700 flex flex-col sm:top-9 h-[calc(100vh-30px)] shadow-2xl"
        :class="[
          position === 'left' ? 'left-0 border-r' : 'right-0 border-l',
          widthClass,
          isMobile ? 'top-[30px] w-full' : 'top-[30px]',
        ]"
      >
        <!-- Header -->
        <div
          class="flex items-center justify-between border-b border-gray-700 shrink-0"
          :class="isMobile ? 'px-3 py-2' : 'px-4 py-2'"
        >
          <div class="flex items-center space-x-3">
            <div
              v-if="icon"
              class="flex items-center justify-center rounded-lg"
              :class="[iconBackground, isMobile ? 'w-7 h-7' : 'w-8 h-8']"
            >
              <component
                :is="icon"
                :size="isMobile ? 18 : 20"
                :class="iconColor"
              />
            </div>
            <h2
              class="font-semibold text-white"
              :class="isMobile ? 'text-base' : 'text-lg'"
            >
              {{ title }}
            </h2>
          </div>
          <Button
            variant="ghost"
            :icon="X"
            :size="isMobile ? 'sm' : 'md'"
            @click="close"
          />
        </div>

        <!-- Header Action -->
        <div
          v-if="$slots.headerAction"
          class="border-b border-gray-700 shrink-0"
          :class="isMobile ? 'p-3' : 'p-4'"
        >
          <slot name="headerAction" />
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-auto">
          <slot />
        </div>

        <!-- Footer -->
        <div
          v-if="$slots.footer"
          class="border-t border-gray-700 shrink-0"
          :class="isMobile ? 'p-3' : 'p-4'"
        >
          <slot name="footer" />
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
import { useWindowSize } from "../../composables/useWindowSize";

const { isMobile } = useWindowSize();

interface DrawerProps {
  id: string;
  visible?: boolean;
  title?: string;
  icon?: Component;
  iconBackground?: string;
  iconColor?: string;
  position?: "left" | "right";
  width?: "sm" | "md" | "lg" | "xl" | "2xl";
  closeOnOverlay?: boolean;
  closeOnEsc?: boolean;
  parentId?: string;
  onBeforeOpen?: () => void | Promise<void>;
  onOpened?: () => void;
  onBeforeClose?: () => boolean | Promise<boolean>;
  onClosed?: () => void;
  onError?: (error: Error) => void;
}

const props = withDefaults(defineProps<DrawerProps>(), {
  visible: false,
  title: "",
  position: "left",
  width: "md",
  iconBackground: "bg-gray-700",
  iconColor: "text-gray-300",
  closeOnOverlay: true,
  closeOnEsc: true,
});

const emit = defineEmits<{
  "update:visible": [visible: boolean];
  close: [];
}>();

const {
  overlayStore,
  registerOverlay,
  unregisterOverlay,
  closeOverlay,
  isOverlayTransitioning,
} = useOverlay();

const widthClass = computed(() => {
  if (isMobile.value) {
    return "w-full";
  }

  const widthMap = {
    sm: "w-80 sm:w-96",
    md: "w-96 lg:w-[28rem]",
    lg: "w-[32rem]",
    xl: "w-[36rem]",
    "2xl": "w-[42rem]",
  };
  return widthMap[props.width];
});

const isVisible = computed(() => overlayStore.isVisible(props.id));
const isTransitioning = computed(() => isOverlayTransitioning(props.id));

const close = async (): Promise<void> => {
  if (isTransitioning.value) {
    console.warn(`⚠️ Drawer ${props.id} is transitioning, ignoring close`);
    return;
  }

  await closeOverlay(props.id);
  emit("update:visible", false);
  emit("close");
};

const handleOverlayClick = (): void => {
  if (props.closeOnOverlay && !isTransitioning.value) {
    close();
  }
};

/**
 * Handle keyboard events - close drawer on Esc key
 */
const handleKeydown = (event: KeyboardEvent): void => {
  if (
    event.key === "Escape" &&
    props.closeOnEsc &&
    isVisible.value &&
    !isTransitioning.value
  ) {
    close();
  }
};

onMounted(() => {
  registerOverlay({
    id: props.id,
    type: "drawer",
    parentId: props.parentId || null,
    title: props.title,
    icon: props.icon,
    props: {
      position: props.position,
      width: props.width,
      iconBackground: props.iconBackground,
      iconColor: props.iconColor,
      closeOnOverlay: props.closeOnOverlay,
      closeOnEsc: props.closeOnEsc,
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
  document.removeEventListener("keydown", handleKeydown);
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
    document.addEventListener("keydown", handleKeydown);
  } else {
    document.removeEventListener("keydown", handleKeydown);
  }
});
</script>
