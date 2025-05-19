<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface MemeItem {
  id: string;
  url: string;
  description?: string;
}

interface UserPreferences {
  copy_to_clipboard: boolean;
}

const searchText = ref('');
const memeResults = ref<MemeItem[]>([]);
const isLoading = ref(false);
const errorMessage = ref('');
const preferences = ref<UserPreferences>({
  copy_to_clipboard: true
});
// 添加选中表情的状态
const selectedMeme = ref<MemeItem | null>(null);

// Load user preferences
onMounted(async () => {
  try {
    preferences.value = await invoke('get_user_preferences');
  } catch (error) {
    console.error('Failed to load preferences:', error);
  }
});

// 监听selectedMeme变化，3秒后自动清除
watch(selectedMeme, (val) => {
  if (val) {
    setTimeout(() => {
      selectedMeme.value = null;
    }, 3000);
  }
});

// 改进搜索函数，增加错误恢复机制
const searchMemes = async () => {
  if (!searchText.value.trim()) return;
  
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    const searchTimeout = setTimeout(() => {
      if (isLoading.value) {
        errorMessage.value = "搜索请求超时，请检查网络连接并重试";
        isLoading.value = false;
      }
    }, 15000); // 15秒超时
    
    memeResults.value = await invoke('search_memes', { keyword: searchText.value });
    clearTimeout(searchTimeout);
  } catch (error) {
    console.error('Search error:', error);
    errorMessage.value = `搜索失败: ${error}`;
    
    // 如果搜索失败，等待几秒后清除错误消息
    setTimeout(() => {
      if (errorMessage.value.includes(`搜索失败: ${error}`)) {
        errorMessage.value = '';
      }
    }, 5000);
  } finally {
    isLoading.value = false;
  }
};

// Handle number key presses to select memes
const handleKeyPress = (event: KeyboardEvent) => {
  // Only handle number keys 1-9
  const num = parseInt(event.key);
  if (isNaN(num) || num < 1 || num > 9 || num > memeResults.value.length) {
    return;
  }
  
  // Prevent default behavior (like typing the number)
  event.preventDefault();
  
  // Select the meme at the index (subtract 1 because arrays are 0-indexed)
  selectMeme(memeResults.value[num - 1]);
};

// 改进选择表情处理，增加错误恢复和用户反馈
const selectMeme = async (meme: MemeItem) => {
  try {
    // const originalErrorMessage = errorMessage.value;
    errorMessage.value = '';
    
    // 显示一个临时的选择指示器
    const tempStatus = document.createElement('div');
    tempStatus.className = 'temp-selection-status';
    tempStatus.textContent = '正在处理...';
    document.body.appendChild(tempStatus);
    
    // 设置处理超时
    const processTimeout = setTimeout(() => {
      tempStatus.textContent = '处理超时，请重试';
      tempStatus.classList.add('error');
      setTimeout(() => {
        document.body.removeChild(tempStatus);
      }, 2000);
    }, 10000); // 10秒超时
    
    // Copy to clipboard if enabled
    await invoke('copy_image_to_clipboard', { 
      imageUrl: meme.url,
      window: null // Tauri will automatically provide the window object
    });
    
    clearTimeout(processTimeout);
    tempStatus.textContent = '已选择表情';
    tempStatus.classList.add('success');
    
    // 2秒后移除状态指示
    setTimeout(() => {
      document.body.removeChild(tempStatus);
    }, 2000);
    
    // 设置选中的表情
    selectedMeme.value = meme;
    
    // Emit a custom event to notify parent components about the selection
    const event = new CustomEvent('meme-selected', { detail: meme });
    window.dispatchEvent(event);
  } catch (error) {
    console.error('Failed to process meme selection:', error);
    errorMessage.value = `无法选择表情: ${error}`;
    
    // 5秒后清除错误消息
    setTimeout(() => {
      errorMessage.value = '';
    }, 5000);
  }
};

// Setup keyboard listeners

onMounted(async () => {
  // Add keyboard event listener for number keys
  window.addEventListener('keydown', handleKeyPress);
});

onUnmounted(() => {
  // Clean up event listeners
  window.removeEventListener('keydown', handleKeyPress);
});

// Toggle clipboard functionality
const toggleClipboard = async () => {
  try {
    preferences.value.copy_to_clipboard = !preferences.value.copy_to_clipboard;
    await invoke('set_copy_to_clipboard', { 
      enabled: preferences.value.copy_to_clipboard 
    });
  } catch (error) {
    console.error('Failed to update preferences:', error);
  }
};
</script>

<template>
  <div class="meme-selector">
    <!-- 添加选中表情的展示部分 -->
    <div v-if="selectedMeme" class="selected-meme">
      <h2>已选择的表情</h2>
      <img :src="selectedMeme.url" :alt="selectedMeme.description || '已选择的表情'" />
      <p>图片已复制到剪贴板！</p>
      <p>现在您可以将其粘贴到任何应用程序中。</p>
    </div>
    
    <div class="search-container">
      <input 
        v-model="searchText"
        @keydown.enter="searchMemes"
        placeholder="🔍搜索表情包"
        type="text"
      />
      <button @click="searchMemes" :disabled="isLoading">
        {{ isLoading ? 'Searching...' : 'Search' }}
      </button>
    </div>
    
    <div class="preferences">
      <label>
        <input 
          type="checkbox" 
          :checked="preferences.copy_to_clipboard"
          @change="toggleClipboard"
        />
        复制到剪贴板📋
      </label>
    </div>
    
    <div v-if="errorMessage" class="error-message">
      {{ errorMessage }}
      <button class="close-button" @click="errorMessage = ''">×</button>
    </div>
    
    <div class="meme-grid">
      <div 
        v-for="(meme, index) in memeResults" 
        :key="meme.id"
        class="meme-item"
        @click="selectMeme(meme)"
      >
        <div class="meme-number">{{ index + 1 }}</div>
        <img :src="meme.url" :alt="meme.description || 'Meme image'" />
      </div>
    </div>
    
    <!-- 添加使用说明 -->
    <div class="instructions">
      <h3>使用说明：</h3>
      <ol>
        <li>在搜索框中输入关键词并按下回车</li>
        <li>浏览表情搜索结果</li>
        <li>按数字键（1-9）选择表情</li>
        <li>所选表情将被复制到剪贴板</li>
        <li>将表情粘贴到任何应用程序中！</li>
      </ol>
    </div>
  </div>
</template>

<style scoped>
.meme-selector {
  width: 100%;
  max-width: 1100px;
  margin: 0 auto;
  padding: 1rem;
}

.search-container {
  display: flex;
  margin-bottom: 1rem;
}

.search-container input {
  flex: 1;
  padding: 0.5rem;
  font-size: 1rem;
  border: 1px solid #ccc;
  border-radius: 4px 0 0 4px;
}

.search-container button {
  padding: 0.5rem 1rem;
  background-color: #4a8fe7;
  color: white;
  border: none;
  border-radius: 0 4px 4px 0;
  cursor: pointer;
}

.search-container button:disabled {
  background-color: #a0b8d8;
}

.preferences {
  margin-bottom: 1rem;
  padding: 0.5rem;
  background-color: #f5f5f5;
  border-radius: 4px;
}

.error-message {
  position: relative;
  color: #d32f2f;
  margin-bottom: 1rem;
  padding: 0.5rem 2rem 0.5rem 0.5rem;
  background-color: #ffebee;
  border-radius: 4px;
}

.error-message .close-button {
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

.error-message .close-button:hover {
  opacity: 1;
}

.meme-grid {
  display: grid;
  /* grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); */
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1rem;
}

.meme-item {
  position: relative;
  border: 1px solid #ddd;
  border-radius: 4px;
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.2s;
}

.meme-item:hover {
  transform: scale(1.05);
}

.meme-number {
  position: absolute;
  top: 5px;
  left: 5px;
  background-color: rgba(0, 0, 0, 0.7);
  color: white;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
}

.meme-item img {
  width: 100%;
  height: auto;
  display: block;
}

/* 为临时状态指示器添加样式 */
:global(.temp-selection-status) {
  position: fixed;
  bottom: 20px;
  right: 20px;
  padding: 0.75rem 1.5rem;
  background-color: #f5f5f5;
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  z-index: 1000;
  animation: fadeIn 0.3s;
}

:global(.temp-selection-status.success) {
  background-color: #e8f5e9;
  color: #1b5e20;
}

:global(.temp-selection-status.error) {
  background-color: #ffebee;
  color: #b71c1c;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* 添加选中表情的样式 */
.selected-meme {
  margin: 2rem 0;
  padding: 1rem;
  background-color: #f0f8ff;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  text-align: center;
}

.selected-meme img {
  max-width: 300px;
  max-height: 300px;
  margin: 1rem auto;
  display: block;
}

/* 添加使用说明样式 */
.instructions {
  text-align: left;
  margin-top: 2rem;
  padding: 1rem;
  background-color: #f5f5f5;
  border-radius: 8px;
}
</style>
