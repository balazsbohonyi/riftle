/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare module "vue-virtual-scroller" {
  import type { DefineComponent } from "vue";
  export const RecycleScroller: DefineComponent<{
    items: unknown[];
    itemSize: number;
    keyField?: string;
    direction?: string;
    minItemSize?: number;
    sizeField?: string;
    typeField?: string;
    buffer?: number;
    pageMode?: boolean;
    prerender?: number;
    emitUpdate?: boolean;
    updateInterval?: number;
    listTag?: string;
    itemTag?: string;
  }, {}, any>;
  export const DynamicScroller: DefineComponent<Record<string, unknown>, {}, any>;
  export const DynamicScrollerItem: DefineComponent<Record<string, unknown>, {}, any>;
}
