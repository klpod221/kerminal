<template>
  <div class="relative inline-block">
    <!-- Trigger Element -->
    <div
      ref="triggerRef"
      @click.stop="handleTriggerClick"
      @mouseenter="handleMouseEnter"
      @mouseleave="handleMouseLeave"
    >
      <slot />
    </div>

    <!-- Backdrop -->
    <Teleport to="body">
      <div
        v-if="visible"
        class="fixed inset-0 z-40"
        @click="handleBackdropClick"
      />
    </Teleport>

    <!-- PopConfirm Overlay -->
    <Teleport to="body">
      <div
        v-if="visible"
        ref="popconfirmRef"
        :style="popconfirmStyle"
        class="fixed z-50"
      >
        <Transition
          enter-active-class="transition-all duration-200 ease-out"
          enter-from-class="opacity-0 scale-95 translate-y-2"
          enter-to-class="opacity-100 scale-100 translate-y-0"
          leave-active-class="transition-all duration-150 ease-in"
          leave-from-class="opacity-100 scale-100 translate-y-0"
          leave-to-class="opacity-0 scale-95 translate-y-2"
        >
          <div
            v-if="visible"
            class="bg-[#1a1a1a] border border-gray-700 rounded-lg shadow-2xl p-4 min-w-[280px] max-w-[400px]"
            @click.stop
          >
            <!-- Header with Icon and Title -->
            <div class="flex items-start space-x-3 mb-4">
              <div class="flex-shrink-0 mt-0.5">
                <div class="rounded-lg p-2" :class="iconBackgroundClass">
                  <component
                    :is="iconComponent"
                    :size="16"
                    :class="iconClass"
                  />
                </div>
              </div>
              <div class="flex-1 min-w-0">
                <div
                  v-if="title"
                  class="text-sm font-semibold text-white mb-1.5"
                >
                  {{ title }}
                </div>
                <div class="text-sm text-gray-300 leading-5">
                  {{ content }}
                </div>
              </div>
            </div>

            <!-- Action Buttons -->
            <div class="flex justify-end space-x-2">
              <Button
                variant="outline"
                size="sm"
                :text="cancelText"
                @click="handleCancel"
              />
              <Button
                :variant="buttonVariant"
                size="sm"
                :text="okText"
                @click="handleConfirm"
              />
            </div>
          </div>
        </Transition>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from "vue";
import { AlertTriangle, HelpCircle } from "lucide-vue-next";
import Button from "./Button.vue";

interface PopConfirmProps {
  title?: string;
  content: string;
  okText?: string;
  cancelText?: string;
  okType?: "primary" | "danger";
  trigger?: "click" | "hover";
  placement?:
    | "top"
    | "bottom"
    | "left"
    | "right"
    | "topLeft"
    | "topRight"
    | "bottomLeft"
    | "bottomRight";
  disabled?: boolean;
}

const props = withDefaults(defineProps<PopConfirmProps>(), {
  okText: "OK",
  cancelText: "Cancel",
  okType: "primary",
  trigger: "click",
  placement: "top",
  disabled: false,
});

const emit = defineEmits<{
  confirm: [];
  cancel: [];
  visibleChange: [visible: boolean];
}>();

// Refs
const triggerRef = ref<HTMLElement>();
const popconfirmRef = ref<HTMLElement>();
const visible = ref(false);
const popconfirmStyle = ref({});

// Computed
const iconComponent = computed(() => {
  return props.okType === "danger" ? AlertTriangle : HelpCircle;
});

const iconClass = computed(() => {
  return props.okType === "danger" ? "text-red-400" : "text-blue-400";
});

const iconBackgroundClass = computed(() => {
  return props.okType === "danger" ? "bg-red-500/20" : "bg-blue-500/20";
});

const buttonVariant = computed(() => {
  return props.okType === "danger" ? "danger" : "primary";
});

// Methods
/**
 * Calculate the optimal position for the popconfirm
 */
const calculatePosition = async (): Promise<void> => {
  if (!triggerRef.value || !popconfirmRef.value) return;

  await nextTick();

  const triggerRect = triggerRef.value.getBoundingClientRect();
  const popconfirmRect = popconfirmRef.value.getBoundingClientRect();
  const viewport = {
    width: window.innerWidth,
    height: window.innerHeight,
  };

  let top = 0;
  let left = 0;
  const offset = 8;

  switch (props.placement) {
    case "top":
      top = triggerRect.top - popconfirmRect.height - offset;
      left = triggerRect.left + (triggerRect.width - popconfirmRect.width) / 2;
      break;
    case "bottom":
      top = triggerRect.bottom + offset;
      left = triggerRect.left + (triggerRect.width - popconfirmRect.width) / 2;
      break;
    case "left":
      top = triggerRect.top + (triggerRect.height - popconfirmRect.height) / 2;
      left = triggerRect.left - popconfirmRect.width - offset;
      break;
    case "right":
      top = triggerRect.top + (triggerRect.height - popconfirmRect.height) / 2;
      left = triggerRect.right + offset;
      break;
    case "topLeft":
      top = triggerRect.top - popconfirmRect.height - offset;
      left = triggerRect.left;
      break;
    case "topRight":
      top = triggerRect.top - popconfirmRect.height - offset;
      left = triggerRect.right - popconfirmRect.width;
      break;
    case "bottomLeft":
      top = triggerRect.bottom + offset;
      left = triggerRect.left;
      break;
    case "bottomRight":
      top = triggerRect.bottom + offset;
      left = triggerRect.right - popconfirmRect.width;
      break;
  }

  // Keep popconfirm within viewport bounds
  const padding = 12;
  if (left < padding) left = padding;
  if (left + popconfirmRect.width > viewport.width - padding) {
    left = viewport.width - popconfirmRect.width - padding;
  }
  if (top < padding) top = padding;
  if (top + popconfirmRect.height > viewport.height - padding) {
    top = viewport.height - popconfirmRect.height - padding;
  }

  popconfirmStyle.value = {
    top: `${Math.round(top)}px`,
    left: `${Math.round(left)}px`,
  };
};

/**
 * Show the popconfirm
 */
const show = async (): Promise<void> => {
  if (props.disabled) return;

  visible.value = true;
  emit("visibleChange", true);
  await nextTick();
  await calculatePosition();
};

/**
 * Hide the popconfirm
 */
const hide = (): void => {
  visible.value = false;
  emit("visibleChange", false);
};

const handleTriggerClick = (): void => {
  if (props.trigger === "click") {
    if (visible.value) {
      hide();
    } else {
      show();
    }
  }
};

let hoverTimeout: number | null = null;

const handleMouseEnter = (): void => {
  if (props.trigger === "hover") {
    if (hoverTimeout) {
      window.clearTimeout(hoverTimeout);
      hoverTimeout = null;
    }
    show();
  }
};

const handleMouseLeave = (): void => {
  if (props.trigger === "hover") {
    hoverTimeout = window.setTimeout(() => {
      if (props.trigger === "hover") {
        hide();
      }
    }, 100);
  }
};

const handleConfirm = (): void => {
  emit("confirm");
  hide();
};

const handleCancel = (): void => {
  emit("cancel");
  hide();
};

const handleBackdropClick = (): void => {
  hide();
};

const handleKeydown = (event: KeyboardEvent): void => {
  if (visible.value && event.key === "Escape") {
    handleCancel();
  }
};

// Lifecycle
onMounted(() => {
  document.addEventListener("keydown", handleKeydown);
  window.addEventListener("resize", calculatePosition);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("resize", calculatePosition);
  if (hoverTimeout) {
    window.clearTimeout(hoverTimeout);
  }
});

// Expose methods for programmatic control
defineExpose({
  show,
  hide,
});
</script>
