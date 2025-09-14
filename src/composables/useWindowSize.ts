import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function useWindowSize() {
  const width = ref(0);
  const height = ref(0);

  // Function to update window size
  const updateSize = async () => {
    const window = getCurrentWindow();
    const size = await window.innerSize();

    width.value = size.width;
    height.value = size.height;
  };

  onMounted(() => {
    updateSize();
    window.addEventListener('resize', updateSize);
  });

  onUnmounted(() => {
    window.removeEventListener('resize', updateSize);
  });

  return { width, height };
}
