<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

const menuItems = [
  { path: '/review', label: '复习' },
  { path: '/questions', label: '题目' },
  { path: '/stats', label: '统计' }
]
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-title">错题本</div>
    <nav class="sidebar-nav">
      <RouterLink
        v-for="item in menuItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ active: route.path.startsWith(item.path) }"
      >
        {{ item.label }}
      </RouterLink>
    </nav>
  </aside>
</template>

<style scoped>
.sidebar {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  width: 200px;
  background-color: #ffffff;
  color: #333;
  display: flex;
  flex-direction: column;
  z-index: 100;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.1);
}

.sidebar-title {
  padding: 20px;
  font-size: 18px;
  font-weight: bold;
  border-bottom: 1px solid #e0e0e0;
  color: #333;
}

.sidebar-nav {
  flex: 1;
  padding-top: 20px;
}

.nav-item {
  display: block;
  padding: 12px 20px;
  color: #333;
  text-decoration: none;
  transition: all 0.2s;
  border-left: 3px solid transparent;
}

.nav-item:hover {
  background-color: #f5f5f5;
}

.nav-item.active {
  background-color: #f5f5f5;
  border-left-color: #4CAF50;
  color: #4CAF50;
  font-weight: 500;
}
</style>
