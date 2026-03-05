<script setup lang="ts">
// Phase 1: minimal launcher shell
// Full launcher UI implemented in Phase 5 (Launcher Window UI)
</script>

<template>
  <div id="app">
    <!-- Launcher UI placeholder — implemented in Phase 5 -->
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  /* Required for transparent window: without this, the launcher shows a white rectangle */
  background: transparent;
  overflow: hidden;
}

#app {
  width: 100%;
  height: 100%;
  background: #1c1c1e;
  border-radius: 12px;
}
</style>
