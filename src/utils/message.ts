import { createVNode, render } from "vue";
import MessageContainer from "../components/ui/MessageContainer.vue";
import { ask } from "@tauri-apps/plugin-dialog";

export interface MessageOptions {
  type?: "success" | "error" | "warning" | "info" | "loading";
  title?: string;
  content: string;
  duration?: number;
  closable?: boolean;
}

let messageContainer: any = null;

const getMessageContainer = () => {
  if (messageContainer) return messageContainer;

  const container = document.createElement("div");
  document.body.appendChild(container);

  const vnode = createVNode(MessageContainer);
  render(vnode, container);

  messageContainer = vnode.component?.exposed;
  return messageContainer;
};

class MessageService {
  private create(options: MessageOptions): Promise<void> {
    return new Promise((resolve) => {
      const container = getMessageContainer();
      if (container) {
        container.add({
          ...options,
          onClose: () => {
            resolve();
          },
        });
      } else {
        resolve();
      }
    });
  }

  success(content: string, title?: string, duration = 3000): Promise<void> {
    return this.create({
      type: "success",
      content,
      title,
      duration,
    });
  }

  error(content: string, title?: string, duration = 4000): Promise<void> {
    return this.create({
      type: "error",
      content,
      title,
      duration,
    });
  }

  warning(content: string, title?: string, duration = 3000): Promise<void> {
    return this.create({
      type: "warning",
      content,
      title,
      duration,
    });
  }

  info(content: string, title?: string, duration = 3000): Promise<void> {
    return this.create({
      type: "info",
      content,
      title,
      duration,
    });
  }

  loading(content: string, title?: string): Promise<void> {
    return this.create({
      type: "loading",
      content,
      title,
      duration: 0,
      closable: false,
    });
  }

  destroy(): void {
    // Current implementation does not support destroying the singleton container
    // If needed, we could expose a clear method on MessageContainer
    if (messageContainer) {
      // For now, do nothing or we could potentially re-render empty
    }
  }
}

export const message = new MessageService();

export function showSuccess(content: string, title?: string): Promise<void> {
  return message.success(content, title);
}

export function showError(content: string, title?: string): Promise<void> {
  return message.error(content, title);
}

export function showWarning(content: string, title?: string): Promise<void> {
  return message.warning(content, title);
}

export function showInfo(content: string, title?: string): Promise<void> {
  return message.info(content, title);
}

export async function showConfirm(
  title: string,
  content: string,
): Promise<boolean> {
  return await ask(content, { title, kind: "warning" });
}
