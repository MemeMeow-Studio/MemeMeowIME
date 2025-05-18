<script setup lang="ts">
import { ref, onMounted, onErrorCaptured, watch } from 'vue';
import MemeSelector from './components/MemeSelector.vue';
import Settings from './components/Settings.vue';
import { listen } from '@tauri-apps/api/event';

const selectedMeme = ref<{ id: string; url: string; description?: string } | null>(null);
const showSettings = ref(false);
const notification = ref<{ type: 'error' | 'success', message: string } | null>(null);
const fatalError = ref<string | null>(null);

// 全局错误处理
onErrorCaptured((err, instance, info) => {
  console.error('Vue错误被捕获:', err, instance, info);
  showNotification('error', `应用发生错误: ${err}`);
  return true; // 防止错误继续向上传播
});

// 监听全局未捕获的promise错误
window.addEventListener('unhandledrejection', (event) => {
  console.error('未处理的Promise拒绝:', event.reason);
  showNotification('error', `未处理的异步错误: ${event.reason?.message || '未知错误'}`);
});

// 监听全局JavaScript错误
window.addEventListener('error', (event) => {
  console.error('全局JavaScript错误:', event.error);
  
  // 检查是否是致命错误
  if (event.error?.toString().includes('致命错误') ||
      (typeof event.error === 'object' && event.error?.isFatal)) {
    fatalError.value = `应用发生致命错误: ${event.error?.message || event.message || '未知错误'}`;
  } else {
    showNotification('error', `JavaScript错误: ${event.error?.message || event.message || '未知错误'}`);
  }
});

// Handle meme selection
onMounted(() => {
  window.addEventListener('meme-selected', ((event: CustomEvent) => {
    selectedMeme.value = event.detail;
  }) as EventListener);
  
  // 监听快捷键注册失败事件
  listen('shortcut-registration-failed', (event) => {
    showNotification('error', `${event.payload}`);
  });
  
  // 添加自动恢复机制
  setInterval(() => {
    const uiElement = document.querySelector('.container');
    if (uiElement && getComputedStyle(uiElement).display === 'none') {
      console.warn('检测到UI可能已被隐藏，尝试恢复...');
      // 尝试恢复UI
      if (uiElement instanceof HTMLElement) {
        uiElement.style.display = 'block';
      }
    }
  }, 5000);
});

// 监听selectedMeme变化，3秒后自动清除
watch(selectedMeme, (val) => {
  if (val) {
    setTimeout(() => {
      selectedMeme.value = null;
    }, 3000);
  }
});

const toggleSettings = () => {
  showSettings.value = !showSettings.value;
};

const showNotification = (type: 'error' | 'success', message: string) => {
  notification.value = { type, message };
  setTimeout(() => {
    notification.value = null;
  }, 5000);
};

// 添加重置应用功能
const resetApp = () => {
  fatalError.value = null;
  window.location.reload();
};
</script>

<template>
  <div class="container">
    <!-- 致命错误显示 -->
    <div v-if="fatalError" class="fatal-error">
      <h2>应用发生问题</h2>
      <p>{{ fatalError }}</p>
      <p>请尝试重新启动应用或联系技术支持。</p>
      <button @click="resetApp" class="reset-button">重置应用</button>
    </div>
    
    <div v-else>
      <div v-if="notification" class="notification" :class="notification.type">
        {{ notification.message }}
        <button class="close-button" @click="notification = null">×</button>
      </div>
      
      <header class="app-header">
        <h1>Meme Input</h1>
        <button class="settings-button" @click="toggleSettings">
          <span v-if="showSettings">返回</span>
          <span v-else>设置</span>
        </button>
      </header>
      
      <div v-if="!showSettings">
        <div v-if="selectedMeme" class="selected-meme">
          <h2>Selected Meme</h2>
          <img :src="selectedMeme.url" :alt="selectedMeme.description || 'Selected meme'" />
          <p>The image has been copied to your clipboard!</p>
          <p>You can now paste it into any application.</p>
        </div>
        
        <MemeSelector />
        
        <div class="instructions">
          <h3>Instructions:</h3>
          <ol>
            <li>Type keywords in the search box and press Enter</li>
            <li>Browse through the meme results</li>
            <li>Press number keys (1-9) to select a meme</li>
            <li>The selected meme will be copied to your clipboard</li>
            <li>Paste the meme into any application!</li>
          </ol>
        </div>
      </div>
      
      <Settings v-if="showSettings" />
    </div>
  </div>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  line-height: 1.5;
  font-weight: 400;
  color: #213547;
  background-color: #ffffff;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
  display: flex;
  min-width: 320px;
  min-height: 100vh;
}

#app {
  width: 100%;
  margin: 0 auto;
  text-align: center;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.settings-button {
  padding: 0.5rem 1rem;
  background-color: #f0f0f0;
  border: 1px solid #ccc;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
}

.settings-button:hover {
  background-color: #e0e0e0;
}

.container {
  width: 100%;
  max-width: 1000px;
  margin: 0 auto;
  padding: 2rem;
}

.selected-meme {
  margin: 2rem 0;
  padding: 1rem;
  background-color: #f0f8ff;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.selected-meme img {
  max-width: 300px;
  max-height: 300px;
  margin: 1rem auto;
  display: block;
}

.instructions {
  text-align: left;
  margin-top: 2rem;
  padding: 1rem;
  background-color: #f5f5f5;
  border-radius: 8px;
}

.notification {
  position: fixed;
  top: 20px;
  right: 20px;
  padding: 1rem 2rem 1rem 1rem;
  border-radius: 4px;
  z-index: 1000;
  animation: fadeIn 0.3s;
  max-width: 400px;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.notification .close-button {
  position: absolute;
  top: 5px;
  right: 5px;
  background: transparent;
  border: none;
  font-size: 1rem;
  cursor: pointer;
  color: inherit;
  opacity: 0.7;
}

.notification .close-button:hover {
  opacity: 1;
}

.notification.error {
  background-color: #ffebee;
  border-left: 4px solid #f44336;
  color: #b71c1c;
}

.notification.success {
  background-color: #e8f5e9;
  border-left: 4px solid #4caf50;
  color: #1b5e20;
}

.fatal-error {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background-color: #fff;
  border: 1px solid #f44336;
  border-radius: 8px;
  padding: 2rem;
  max-width: 80%;
  text-align: center;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
  z-index: 9999;
}

.fatal-error h2 {
  color: #d32f2f;
  margin-top: 0;
}

.reset-button {
  padding: 0.75rem 2rem;
  background-color: #4caf50;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  margin-top: 1rem;
}

.reset-button:hover {
  background-color: #388e3c;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-20px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes fadeOut {
  from { opacity: 1; }
  to { opacity: 0; }
}
</style>