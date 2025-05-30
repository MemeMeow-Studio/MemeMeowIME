<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface CommunityInfo {
  // resource_url: string;
  // update_url: string;
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

interface ApiUrl {
  name: string;
  url: string;
}

interface ApiUrlConfig {
  urls: ApiUrl[];
  active_index: number;
}

const communityManifest = ref<CommunityManifest | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);
const refreshing = ref(false);
const enabledLibs = ref<string[]>([]);
const processingLibs = reactive(new Set<string>());

// API URL管理
const apiConfig = ref<ApiUrlConfig>({
  urls: [{ name: "默认API", url: "https://mememeow.morami.icu" }],
  active_index: 0
});
const apiServerConfig = ref<ApiUrl[]>(
  [{ name: "默认API", url: "https://mememeow.morami.icu" }]
);
const showApiManager = ref(false);
const newApiName = ref("");
const newApiUrl = ref("");
// const apiSaving = ref(false);

// 将时间戳转换为可读格式
const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString('zh-CN', { 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric'
  });
};

// 加载API配置
const loadApiConfig = async () => {
  try {
    const config = await invoke<ApiUrlConfig>('get_api_url_config');
    apiConfig.value = config;
  } catch (err) {
    console.error('加载API配置失败:', err);
    error.value = `加载API配置失败: ${err}`;
  }
};
// 加载API Server的所有可能API
const loadApiServerConfig = async () => {
  try {
    var config = await invoke<ApiUrl[]>('get_api_server_urls_config');
    // console.log('加载API Server urls:', config);
    apiServerConfig.value = config;
  } catch (err) {
    console.error('加载API Server urls失败:', err);
    error.value = `加载API Server urls失败: ${err}`;
  }
};

// 保存API配置
// const saveApiConfig = async () => {
//   apiSaving.value = true;
//   try {
//     await invoke('update_api_url_config', { config: apiConfig.value });
//   } catch (err) {
//     console.error('保存API配置失败:', err);
//     error.value = `保存API配置失败: ${err}`;
//   } finally {
//     apiSaving.value = false;
//   }
// };

const addNewApi = async (name: string, url: string) => {
  if (!name || !url) {
    error.value = "API名称和地址不能为空";
    return;
  }
  
  try {
    await invoke('add_api_url', { name, url });
    // 重新加载配置
    await loadApiConfig();
  } catch (err) {
    console.error('添加API失败:', err);
    error.value = `添加API失败: ${err}`;
  }
};

// 添加新API
const addNewApiFromInput = async () => {
  if (!newApiName.value || !newApiUrl.value) {
    error.value = "API名称和地址不能为空";
    return;
  }
  
  try {
    addNewApi(newApiName.value, newApiUrl.value);
    
    // 清空输入
    newApiName.value = "";
    newApiUrl.value = "";
  } catch (err) {
    console.error('添加API失败:', err);
    error.value = `添加API失败: ${err}`;
  }
};

// 删除API
const removeApi = async (index: number) => {
  try {
    await invoke('remove_api_url', { index });
    await loadApiConfig();
  } catch (err) {
    console.error('删除API失败:', err);
    error.value = `删除API失败: ${err}`;
  }
};

// 设置活跃API
const setActiveApi = async (index: number) => {
  try {
    await invoke('set_active_api_url', { index });
    apiConfig.value.active_index = index;
  } catch (err) {
    console.error('设置活跃API失败:', err);
    error.value = `设置活跃API失败: ${err}`;
  }
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

// 切换API管理器显示状态
const toggleApiManager = () => {
  showApiManager.value = !showApiManager.value;
  if (showApiManager.value) {
    loadApiConfig();
  }
};

onMounted(() => {
  loadCommunityManifest();
  loadApiServerConfig();
  loadApiConfig();
});
</script>

<template>
  <div class="meme-community">
    <div class="header">
      <h2>表情包社区</h2>
      <div class="header-buttons">
        <button 
          class="api-button" 
          @click="toggleApiManager"
        >
          {{ showApiManager ? '关闭API管理' : 'API管理' }}
        </button>
        <button 
          class="refresh-button" 
          @click="refreshManifest" 
          :disabled="refreshing"
        >
          {{ refreshing ? '刷新中...' : '刷新' }}
        </button>
      </div>
    </div>

    <div v-if="error" class="error">
      <p>{{ error }}</p>
      <button @click="error = null">关闭</button>
    </div>

    <!-- API管理界面 -->
    <div v-if="showApiManager" class="api-manager">
      <h3>管理API地址</h3>
      
      <div class="api-list">
        <div v-for="(api, index) in apiConfig.urls" :key="index" class="api-item">
          <div class="api-info">
            <strong>{{ api.name }}</strong>
            <span class="api-url">{{ api.url }}</span>
          </div>
          <div class="api-actions">
            <button 
              class="api-select-button"
              :class="{ 'active': index === apiConfig.active_index }"
              @click="setActiveApi(index)"
            >
              {{ index === apiConfig.active_index ? '当前使用' : '使用' }}
            </button>
            <button 
              class="api-delete-button" 
              @click="removeApi(index)"
              :disabled="apiConfig.urls.length <= 1"
            >
              删除
            </button>
          </div>
        </div>
      </div>
      
      <div class="add-api-form">
        <h4>添加新API</h4>
        <div class="form-group">
          <label for="apiName">名称:</label>
          <input 
            id="apiName" 
            type="text" 
            v-model="newApiName" 
            placeholder="API名称"
          />
        </div>
        <div class="form-group">
          <label for="apiUrl">URL:</label>
          <input 
            id="apiUrl" 
            type="text" 
            v-model="newApiUrl" 
            placeholder="API地址"
          />
        </div>
        <button @click="addNewApiFromInput" class="add-button">添加API</button>
      </div>

      <div class="api-list">
        <h4>可用社区API列表</h4>
        <div v-for="(api, index) in apiServerConfig" :key="index" class="api-item">
          <div class="api-info">
            <strong>{{ api.name }}</strong>
            <span class="api-url">{{ api.url }}</span>
          </div>
          <div class="api-server-actions">
            <button
            @click="addNewApi(api.name, api.url)" class="add-api-from-server-button">添加到配置
          </button>
          </div>
        </div>
      </div>
      
    </div>

    

    <div v-if="loading" class="loading">
      <p>加载社区表情库...</p>
    </div>

    <div v-else-if="communityManifest" class="community-content">
      <div class="community-info">
        <p>
          <strong>最后更新时间:</strong> 
          {{ formatDate(communityManifest.community_info.timestamp) }}
        </p>
        <p v-if="apiConfig.urls.length > 0">
          <strong>当前使用API:</strong> 
          {{ apiConfig.active_index < apiConfig.urls.length ? 
              apiConfig.urls[apiConfig.active_index].name : '默认' }}
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

.header-buttons {
  display: flex;
  gap: 10px;
}

.refresh-button, .api-button {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.refresh-button {
  background-color: #4a8fe7;
  color: white;
}

.api-button {
  background-color: #6c5ce7;
  color: white;
}

.refresh-button:hover:not(:disabled), .api-button:hover:not(:disabled) {
  opacity: 0.9;
}

.refresh-button:disabled, .api-button:disabled {
  opacity: 0.6;
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
  background-color: #ffebee;
  border: 1px solid #ffcdd2;
  border-radius: 4px;
  margin-bottom: 1rem;
}

.api-manager {
  background-color: #f5f5f5;
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 1.5rem;
  border: 1px solid #e0e0e0;
}

.api-list {
  margin: 1rem 0;
}

.api-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  background-color: white;
  border-radius: 4px;
  margin-bottom: 0.5rem;
  border: 1px solid #e0e0e0;
}

.api-info {
  flex: 1;
  overflow: hidden;
}

.api-url {
  display: block;
  color: #666;
  font-size: 0.85rem;
  margin-top: 0.25rem;
  overflow: hidden;
  text-overflow: ellipsis;
}

.api-actions {
  display: flex;
  gap: 8px;
}

.api-select-button, .api-delete-button, .add-button {
  padding: 0.4rem 0.8rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85rem;
}

.add-api-from-server-button{
  padding: 0.4rem 0.8rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85rem;
}

.api-select-button, .add-api-from-server-button {
  background-color: #4caf50;
  color: white;
}

.api-select-button.active {
  background-color: #2e7d32;
}

.api-delete-button {
  background-color: #f44336;
  color: white;
}

.api-delete-button:disabled {
  background-color: #e0e0e0;
  color: #9e9e9e;
  cursor: not-allowed;
}

.add-api-form {
  background-color: white;
  padding: 1rem;
  border-radius: 4px;
  border: 1px solid #e0e0e0;
}

.form-group {
  margin-bottom: 0.75rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.25rem;
  font-weight: 500;
}

.form-group input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  font-size: 0.9rem;
}

.add-button {
  background-color: #4a8fe7;
  color: white;
  margin-top: 0.5rem;
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
