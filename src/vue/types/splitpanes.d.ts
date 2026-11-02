// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

declare module "splitpanes" {
  import { DefineComponent } from "vue";

  export const Splitpanes: DefineComponent<{
    horizontal?: boolean;
    pushOtherPanes?: boolean;
    dblClickSplitter?: boolean;
    resizerStyle?: object;
    class?: string;
  }>;

  export const Pane: DefineComponent<{
    size?: number;
    minSize?: number;
    maxSize?: number;
    class?: string;
  }>;
}
