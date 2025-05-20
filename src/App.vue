<script setup lang="ts">
import { ref, onMounted, onErrorCaptured } from 'vue';
import MemeSelector from './components/MemeSelector.vue';
import Settings from './components/Settings.vue';
import MemeCommunity from './components/MemeCommunity.vue';
import { listen } from '@tauri-apps/api/event';

const activeTab = ref('memes'); // 默认显示表情选择器
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
    // selectedMeme.value = event.detail;
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

// 切换活动标签页
const switchTab = (tab: string) => {
  activeTab.value = tab;
};

// const toggleSettings = () => {
//   showSettings.value = !showSettings.value;
// };

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
      
      <!-- 新的标签式导航栏 -->
      <header class="app-header">
        <h1>Meme Input</h1>
        <div class="tabs-container">
          <div 
            class="tab" 
            :class="{ active: activeTab === 'memes' }" 
            @click="switchTab('memes')"
          >
            表情浏览
          </div>
          <div 
            class="tab" 
            :class="{ active: activeTab === 'community' }" 
            @click="switchTab('community')"
          >
            社区库
          </div>
          <div 
            class="tab" 
            :class="{ active: activeTab === 'settings' }" 
            @click="switchTab('settings')"
          >
            系统设置
          </div>
        </div>
      </header>
      
      <!-- 标签内容区域 -->
      <div class="tab-content">
        <div v-if="activeTab === 'memes'">
          <MemeSelector />
        </div>
        
        <MemeCommunity v-if="activeTab === 'community'" />
        
        <Settings v-if="activeTab === 'settings'" />
      </div>
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
  flex-direction: column;
  align-items: center;
  margin-bottom: 1.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid #e0e0e0;
}

/* 标签导航样式 */
.tabs-container {
  display: flex;
  width: 100%;
  margin-top: 0.5rem;
  border-bottom: 1px solid #e0e0e0;
}

.tab {
  padding: 0.75rem 1.5rem;
  cursor: pointer;
  transition: all 0.3s ease;
  border-bottom: 2px solid transparent;
  font-weight: 500;
}

.tab:hover {
  background-color: #f5f5f5;
}

.tab.active {
  color: #4a8fe7;
  border-bottom: 2px solid #4a8fe7;
}

/* 标签内容区域 */
.tab-content {
  padding: 1rem 0;
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