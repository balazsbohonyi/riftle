/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare module "vue-virtual-scroller" {
  import type { DefineComponent } from "vue";
  export const RecycleScroller: any;
  export const DynamicScroller: DefineComponent<Record<string, unknown>, {}, any>;
  export const DynamicScrollerItem: DefineComponent<Record<string, unknown>, {}, any>;
}
