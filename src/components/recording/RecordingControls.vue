<template>
  <div v-if="isRecording" class="flex items-center gap-2">
    <div class="flex items-center gap-1.5 px-2 py-1 bg-red-500/10 rounded-md">
      <div class="w-2 h-2 bg-red-500 rounded-full animate-pulse"></div>
      <span class="text-red-500 text-xs font-medium">REC</span>
      <span class="text-red-400 text-xs font-mono tabular-nums">
        {{ formattedTime }}
      </span>
    </div>
    <Button variant="ghost" @click="handleStop" title="Stop recording">
      <Square :size="14" class="text-red-500" fill="currentColor" />
    </Button>
  </div>
  <Button
    v-else
    variant="ghost"
    @click="handleStart"
    title="Start recording"
    class="relative"
  >
    <Circle :size="14" class="text-red-500" />
  </Button>
</template>

<script setup lang="ts">
import { computed, ref, watch, onUnmounted } from "vue";
import { Circle, Square } from "lucide-vue-next";
import { useRecordingStore } from "../../stores/recording";
import Button from "../ui/Button.vue";

interface RecordingControlsProps {
  terminalId: string;
}

const props = defineProps<RecordingControlsProps>();
const recordingStore = useRecordingStore();

const recordingTime = ref(0);
let interval: number | null = null;

const isRecording = computed(() =>
  recordingStore.isRecording(props.terminalId),
);

const formattedTime = computed(() => {
  const minutes = Math.floor(recordingTime.value / 60);
  const seconds = recordingTime.value % 60;
  return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
});

async function handleStart() {
  try {
    await recordingStore.startRecording(props.terminalId, undefined, 80, 24);
    recordingTime.value = 0;
    interval = setInterval(() => {
      recordingTime.value++;
    }, 1000);
  } catch (error) {
    console.error("Failed to start recording:", error);
  }
}

async function handleStop() {
  if (interval) {
    clearInterval(interval);
    interval = null;
  }
  recordingTime.value = 0;

  try {
    await recordingStore.stopRecording(props.terminalId);
  } catch (error) {
    console.error("Failed to stop recording:", error);
  }
}

watch(isRecording, (newVal) => {
  if (!newVal && interval) {
    clearInterval(interval);
    interval = null;
    recordingTime.value = 0;
  }
});

onUnmounted(() => {
  if (interval) {
    clearInterval(interval);
  }
});
</script>
