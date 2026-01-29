<template>
  <div
    class="absolute pointer-events-auto transition-all duration-300 ease-out"
    :style="tooltipStyle"
  >
    <div
      class="bg-bg-tertiary border border-gray-700 rounded-xl shadow-2xl w-[450px] overflow-hidden"
    >
      <!-- Progress bar -->
      <div class="h-1 bg-gray-800">
        <div
          class="h-full bg-linear-to-r from-cyan-500 to-purple-500 transition-all duration-300"
          :style="{ width: `${progress}%` }"
        />
      </div>

      <!-- Content -->
      <div class="p-5">
        <!-- Step indicator -->
        <div class="flex items-center justify-between mb-3">
          <span class="text-xs text-gray-500 font-mono">
            Step {{ stepIndex + 1 }} of {{ totalSteps }}
          </span>
          <button
            v-if="!isLast"
            class="text-xs text-gray-500 hover:text-gray-300 transition-colors"
            @click="$emit('skip')"
          >
            Skip tour
          </button>
        </div>

        <!-- Title -->
        <h3 class="text-lg font-semibold text-white mb-2">
          {{ step.title }}
        </h3>

        <!-- Description -->
        <p class="text-gray-400 text-sm leading-relaxed mb-5">
          {{ step.description }}
        </p>

        <!-- Navigation buttons -->
        <div class="flex items-center justify-between gap-3">
          <Button
            v-if="!isFirst"
            variant="outline"
            size="sm"
            :icon="ChevronLeft"
            text="Previous"
            @click="$emit('prev')"
          />
          <div v-else />

          <Button
            variant="primary"
            size="md"
            :icon="isLast ? Rocket : ChevronRight"
            :icon-right="true"
            :text="isLast ? 'Get Started!' : 'Next'"
            @click="isLast ? $emit('complete') : $emit('next')"
          />
        </div>
      </div>
    </div>

    <!-- Arrow pointer -->
    <div
      v-if="step.position !== 'center' && targetRect"
      class="absolute w-3 h-3 bg-bg-tertiary border-gray-700 transform rotate-45"
      :class="arrowClasses"
      :style="arrowStyle"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { ChevronLeft, ChevronRight, Rocket } from "lucide-vue-next";
import Button from "../ui/Button.vue";
import type { TourStep } from "../../types/tour";

const props = defineProps<{
  step: TourStep;
  stepIndex: number;
  totalSteps: number;
  targetRect: DOMRect | null;
  isFirst: boolean;
  isLast: boolean;
  progress: number;
}>();

defineEmits<{
  next: [];
  prev: [];
  skip: [];
  complete: [];
}>();

const TOOLTIP_OFFSET = 16;
const TOOLTIP_WIDTH = 450;

const tooltipStyle = computed(() => {
  if (!props.targetRect || props.step.position === "center") {
    // Center on screen
    return {
      top: "50%",
      left: "50%",
      transform: "translate(-50%, -50%)",
    };
  }

  const { x, y, width, height } = props.targetRect;
  const position = props.step.position;

  switch (position) {
    case "bottom":
      return {
        top: `${y + height + TOOLTIP_OFFSET}px`,
        left: `${Math.max(16, Math.min(x + width / 2 - TOOLTIP_WIDTH / 2, window.innerWidth - TOOLTIP_WIDTH - 16))}px`,
      };
    case "top":
      return {
        bottom: `${window.innerHeight - y + TOOLTIP_OFFSET}px`,
        left: `${Math.max(16, Math.min(x + width / 2 - TOOLTIP_WIDTH / 2, window.innerWidth - TOOLTIP_WIDTH - 16))}px`,
      };
    case "left":
      return {
        top: `${y + height / 2}px`,
        right: `${window.innerWidth - x + TOOLTIP_OFFSET}px`,
        transform: "translateY(-50%)",
      };
    case "right":
      return {
        top: `${y + height / 2}px`,
        left: `${x + width + TOOLTIP_OFFSET}px`,
        transform: "translateY(-50%)",
      };
    default:
      return {
        top: "50%",
        left: "50%",
        transform: "translate(-50%, -50%)",
      };
  }
});

const arrowClasses = computed(() => {
  const position = props.step.position;

  switch (position) {
    case "bottom":
      return "border-t border-l -top-1.5 left-1/2 -translate-x-1/2";
    case "top":
      return "border-b border-r -bottom-1.5 left-1/2 -translate-x-1/2";
    case "left":
      return "border-t border-r -right-1.5 top-1/2 -translate-y-1/2";
    case "right":
      return "border-b border-l -left-1.5 top-1/2 -translate-y-1/2";
    default:
      return "";
  }
});

const arrowStyle = computed(() => {
  if (!props.targetRect || props.step.position === "center") {
    return { display: "none" };
  }

  const { x, width } = props.targetRect;
  const position = props.step.position;

  if (position === "bottom" || position === "top") {
    // Calculate arrow position based on target element center
    const targetCenter = x + width / 2;
    const tooltipLeft = Math.max(
      16,
      Math.min(
        x + width / 2 - TOOLTIP_WIDTH / 2,
        window.innerWidth - TOOLTIP_WIDTH - 16,
      ),
    );
    const arrowOffset = targetCenter - tooltipLeft;

    return {
      left: `${Math.max(24, Math.min(arrowOffset, TOOLTIP_WIDTH - 24))}px`,
      transform: "translateX(-50%) rotate(45deg)",
    };
  }

  return {};
});
</script>
