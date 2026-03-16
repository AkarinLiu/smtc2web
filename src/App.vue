<template>
  <div id="app">
    <Navbar />
    <main class="page">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
// 组件自动导入，无需手动 import
</script>

<style>
@import './styles/global.css';

#app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
</style>
