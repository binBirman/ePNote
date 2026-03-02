<script setup lang="ts">
import { computed } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import Sidebar from './components/Sidebar.vue'

const route = useRoute()

// 初始化页面不显示侧边栏
const showSidebar = computed(() => route.path !== '/init')
</script>

<template>
  <div class="app">
    <Sidebar v-if="showSidebar" />
    <main class="main-content" :class="{ 'with-sidebar': showSidebar }">
      <RouterView />
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  height: 100%;
  width: 100%;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  background-color: #f5f5f5;
  color: #333;
  font-size: 16px;
}

.app {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.main-content {
  flex: 1;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 30px 40px;
  margin-left: 200px;
}

.main-content.with-sidebar {
  margin-left: 0;
}

button {
  cursor: pointer;
  font-family: inherit;
  font-size: 15px;
}

input, select, textarea {
  font-family: inherit;
  font-size: 15px;
}
</style>
