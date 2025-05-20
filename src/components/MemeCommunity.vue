<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface CommunityInfo {
  resource_url: string;
  update_url: string;
  timestamp: number;
}

interface MemeLib {
  name: string;
  version: string;
  author: string;
  description: string;
  created_at: string;
  timestamp: number;
  tags: string[];
  url: string;
  update_url: string;
  uuid: string;
}

interface CommunityManifest {
  community_info: CommunityInfo;
  meme_libs: Record<string, MemeLib>;
}

const communityManifest = ref<CommunityManifest | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);
const refreshing = ref(false);
const enabledLibs = ref<string[]>([]);
const processingLibs = reactive(new Set<string>());

// 将时间戳转换为可读格式
const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString('zh-CN', { 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric'
  });
};

// 加载社区表情库清单
const loadCommunityManifest = async () => {
  loading.value = true;
  error.value = null;
  
  try {
    const manifest = await invoke<CommunityManifest>('fetch_community_manifest');
    communityManifest.value = manifest;
    // 加载启用的表情库列表
    await loadEnabledLibs();
  } catch (err) {
    console.error('加载社区表情库失败:', err);
    error.value = `加载失败: ${err}`;
  } finally {
    loading.value = false;
  }
};

// 加载已启用的表情库
const loadEnabledLibs = async () => {
  try {
    const enabled = await invoke<string[]>('get_enabled_meme_libs');
    enabledLibs.value = enabled;
  } catch (err) {
    console.error('加载启用状态失败:', err);
    error.value = `加载启用状态失败: ${err}`;
  }
};

// 刷新社区表情库清单
const refreshManifest = async () => {
  if (refreshing.value) return;
  
  refreshing.value = true;
  error.value = null;
  
  try {
    const manifest = await invoke<CommunityManifest>('refresh_community_manifest');
    communityManifest.value = manifest;
    // 刷新后重新加载启用状态
    await loadEnabledLibs();
  } catch (err) {
    console.error('刷新社区表情库失败:', err);
    error.value = `刷新失败: ${err}`;
  } finally {
    refreshing.value = false;
  }
};

// 切换表情库启用状态
const toggleLibEnabled = async (libUuid: string) => {
  if (processingLibs.has(libUuid)) return;
  
  processingLibs.add(libUuid);
  
  try {
    const isCurrentlyEnabled = enabledLibs.value.includes(libUuid);
    
    if (isCurrentlyEnabled) {
      // 禁用表情库
      await invoke('disable_meme_lib', { uuid: libUuid });
      enabledLibs.value = enabledLibs.value.filter(uuid => uuid !== libUuid);
    } else {
      // 启用表情库
      await invoke('enable_meme_lib', { uuid: libUuid });
      enabledLibs.value.push(libUuid);
    }
  } catch (err) {
    console.error(`切换表情库状态失败 ${libUuid}:`, err);
    error.value = `操作失败: ${err}`;
  } finally {
    processingLibs.delete(libUuid);
  }
};

// 检查表情库是否已启用
const isLibEnabled = (libUuid: string): boolean => {
  return enabledLibs.value.includes(libUuid);
};

onMounted(() => {
  loadCommunityManifest();
});
</script>

<template>
  <div class="meme-community">
    <div class="header">
      <h2>表情包社区</h2>
      <button 
        class="refresh-button" 
        @click="refreshManifest" 
        :disabled="refreshing"
      >
        {{ refreshing ? '刷新中...' : '刷新' }}
      </button>
    </div>

    <div v-if="loading" class="loading">
      <p>加载社区表情库...</p>
    </div>

    <div v-else-if="error" class="error">
      <p>{{ error }}</p>
      <button @click="loadCommunityManifest">重试</button>
    </div>

    <div v-else-if="communityManifest" class="community-content">
      <div class="community-info">
        <p>
          <strong>最后更新时间:</strong> 
          {{ formatDate(communityManifest.community_info.timestamp) }}
        </p>
      </div>

      <h3>表情库列表</h3>
      <div class="meme-libs-container">
        <div 
          v-for="lib in Object.values(communityManifest.meme_libs)" 
          :key="lib.uuid"
          class="meme-lib-card"
          :class="{ 'enabled': isLibEnabled(lib.uuid) }"
        >
          <h4>{{ lib.name }}</h4>
          <div class="lib-info">
            <p><strong>版本:</strong> {{ lib.version }}</p>
            <p><strong>作者:</strong> {{ lib.author }}</p>
            <p v-if="lib.description !== 'none'">
              <strong>描述:</strong> {{ lib.description }}
            </p>
            <p><strong>创建时间:</strong> {{ lib.created_at }}</p>
            <p v-if="lib.tags.length > 0">
              <strong>标签:</strong> 
              <span 
                v-for="(tag, index) in lib.tags" 
                :key="index"
                class="tag"
              >
                {{ tag }}
              </span>
            </p>
          </div>
          <div class="lib-actions">
            <button class="view-button">查看详情</button>
            <button 
              class="toggle-button" 
              :class="{ 'enabled': isLibEnabled(lib.uuid) }"
              @click="toggleLibEnabled(lib.uuid)"
              :disabled="processingLibs.has(lib.uuid)"
            >
              {{ isLibEnabled(lib.uuid) ? '已启用' : '启用' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <p>暂无社区表情库数据</p>
      <button @click="refreshManifest">获取数据</button>
    </div>
  </div>
</template>

<style scoped>
.meme-community {
  padding: 1rem;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.refresh-button {
  padding: 0.5rem 1rem;
  background-color: #4a8fe7;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.refresh-button:hover:not(:disabled) {
  background-color: #3a7fd7;
}

.refresh-button:disabled {
  background-color: #a0bfe4;
  cursor: not-allowed;
}

.loading, .error, .empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  text-align: center;
}

.error {
  color: #f44336;
}

.community-info {
  background-color: #f5f5f5;
  padding: 1rem;
  border-radius: 4px;
  margin-bottom: 1rem;
}

.meme-libs-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1rem;
  margin-top: 1rem;
}

.meme-lib-card {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 1rem;
  transition: box-shadow 0.3s, transform 0.3s;
}

.meme-lib-card:hover {
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.meme-lib-card.enabled {
  border-color: #4caf50;
  background-color: #f1f8e9;
}

.lib-info {
  margin: 1rem 0;
}

.lib-info p {
  margin: 0.5rem 0;
}

.tag {
  display: inline-block;
  background-color: #e0f7fa;
  padding: 0.2rem 0.5rem;
  border-radius: 12px;
  font-size: 0.85rem;
  margin-right: 0.5rem;
  margin-bottom: 0.5rem;
}

.lib-actions {
  display: flex;
  justify-content: space-between;
  margin-top: 1rem;
}

.view-button, .toggle-button {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.view-button {
  background-color: #f5f5f5;
  color: #333;
}

.view-button:hover {
  background-color: #e0e0e0;
}

.toggle-button {
  background-color: #4caf50;
  color: white;
}

.toggle-button:hover:not(:disabled) {
  background-color: #388e3c;
}

.toggle-button.enabled {
  background-color: #ff9800;
}

.toggle-button.enabled:hover:not(:disabled) {
  background-color: #f57c00;
}

.toggle-button:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}
</style>
