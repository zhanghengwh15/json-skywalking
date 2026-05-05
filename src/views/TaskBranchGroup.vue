<template>
  <div class="task-branch-group">
    <div class="tool-header">
      <div class="tool-header-left">
        <h1>任务分支管理</h1>
        <p>TB 任务与 Git 分支的关联关系</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-primary" @click="openCreateModal">
          <span class="material-icons">add</span>
          新增
        </button>
      </div>
    </div>

    <div class="filter-bar">
      <input
        v-model="filterKeyword"
        type="text"
        placeholder="按任务ID筛选/TB 名称筛选"
        class="filter-input"
        @keyup.enter="loadList"
      />
      <button class="btn" @click="loadList">
        <span class="material-icons">search</span>
        查询
      </button>
      <button class="btn btn-ghost" @click="clearFilter">
        <span class="material-icons">clear</span>
        重置
      </button>
    </div>

    <div class="content">
      <div class="table-panel">
        <div class="table-wrapper">
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>TB名称</th>
                <th>任务ID</th>
                <th>分支名称</th>
                <th>分组类型</th>
                <th>创建时间</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in items" :key="item.id">
                <td>{{ item.id }}</td>
                <td>{{ item.tbName }}</td>
                <td>{{ item.taskId }}</td>
                <td>{{ item.branchName }}</td>
                <td>
                  <span class="type-tag" :class="item.groupType === 1 ? 'web' : 'back'">
                    {{ item.groupType === 1 ? '前端' : '后端' }}
                  </span>
                </td>
                <td>{{ item.createTime }}</td>
                <td>
                  <div class="action-btns">
                    <button class="icon-btn" title="编辑" @click="openEditModal(item)">
                      <span class="material-icons">edit</span>
                    </button>
                    <button class="icon-btn danger" title="删除" @click="confirmDelete(item)">
                      <span class="material-icons">delete</span>
                    </button>
                  </div>
                </td>
              </tr>
              <tr v-if="items.length === 0">
                <td colspan="7" class="empty-table">暂无数据</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- Modal -->
    <div v-if="showModal" class="modal-overlay" @click.self="closeModal">
      <div class="modal-content">
        <div class="modal-header">
          <h3>{{ isEditing ? '编辑' : '新增' }}任务分支关联</h3>
          <button class="close-btn" @click="closeModal">
            <span class="material-icons">close</span>
          </button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>TB名称</label>
            <input v-model="form.tbName" type="text" placeholder="请输入业务或物理表名称" />
          </div>
          <div class="form-group">
            <label>任务ID</label>
            <input v-model="form.taskId" type="text" placeholder="请输入任务ID" />
          </div>
          <div class="form-group">
            <label>分支名称</label>
            <input v-model="form.branchName" type="text" placeholder="请输入Git分支名称" />
          </div>
          <div class="form-group">
            <label>分组类型</label>
            <select v-model="form.groupType">
              <option :value="1">前端 (web_engineer)</option>
              <option :value="2">后端 (back_engineer)</option>
            </select>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-ghost" @click="closeModal">取消</button>
          <button class="btn btn-primary" @click="saveForm">
            <span class="material-icons">save</span>
            保存
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface TaskBranchGroup {
  id: number
  tbName: string
  taskId: string
  branchName: string
  groupType: number
  createTime: string
  modifyTime: string
  recStatus: number
  createBy: number
  modifyBy: number
}

const items = ref<TaskBranchGroup[]>([])
const filterKeyword = ref('')
const showModal = ref(false)
const isEditing = ref(false)
const editingId = ref(0)

const defaultForm = {
  tbName: '',
  taskId: '',
  branchName: '',
  groupType: 1,
}

const form = ref({ ...defaultForm })

async function loadList() {
  try {
    const keyword = filterKeyword.value.trim() || undefined
    items.value = await invoke<TaskBranchGroup[]>('task_branch_group_list', { keyword })
  } catch (e) {
    console.error('加载列表失败', e)
    alert('加载列表失败: ' + String(e))
  }
}

function clearFilter() {
  filterKeyword.value = ''
  loadList()
}

function openCreateModal() {
  isEditing.value = false
  editingId.value = 0
  form.value = { ...defaultForm }
  showModal.value = true
}

function openEditModal(item: TaskBranchGroup) {
  isEditing.value = true
  editingId.value = item.id
  form.value = {
    tbName: item.tbName,
    taskId: item.taskId,
    branchName: item.branchName,
    groupType: item.groupType,
  }
  showModal.value = true
}

function closeModal() {
  showModal.value = false
}

async function saveForm() {
  if (!form.value.tbName.trim() || !form.value.taskId.trim() || !form.value.branchName.trim()) {
    alert('请填写完整信息')
    return
  }
  try {
    if (isEditing.value) {
      await invoke('task_branch_group_update', {
        id: editingId.value,
        item: {
          tbName: form.value.tbName,
          taskId: form.value.taskId,
          branchName: form.value.branchName,
          groupType: form.value.groupType,
        },
      })
    } else {
      await invoke('task_branch_group_create', {
        item: {
          tbName: form.value.tbName,
          taskId: form.value.taskId,
          branchName: form.value.branchName,
          groupType: form.value.groupType,
          createBy: 0,
        },
      })
    }
    closeModal()
    await loadList()
  } catch (e) {
    console.error('保存失败', e)
    alert('保存失败: ' + String(e))
  }
}

async function confirmDelete(item: TaskBranchGroup) {
  if (!confirm(`确定删除 ID=${item.id} 的记录吗？`)) return
  try {
    await invoke('task_branch_group_delete', { id: item.id })
    await loadList()
  } catch (e) {
    console.error('删除失败', e)
    alert('删除失败: ' + String(e))
  }
}

onMounted(() => {
  loadList()
})
</script>

<style scoped>
.task-branch-group {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 20px 24px 24px;
  gap: 16px;
  overflow: hidden;
}

.tool-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.tool-header-left h1 {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 6px 0;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.tool-header-left p {
  font-size: 14px;
  color: var(--text-muted);
  margin: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.filter-input {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 7px 12px;
  color: var(--text-primary);
  font-size: 13px;
  width: 240px;
  outline: none;
  transition: border-color 0.2s;
}

.filter-input:focus {
  border-color: var(--accent-primary);
}

.filter-input::placeholder {
  color: var(--text-disabled);
}

.content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.table-panel {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--border-default);
  border-top-color: var(--border-hover);
  border-radius: 12px;
  box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.05);
  overflow: hidden;
}

.table-wrapper {
  flex: 1;
  overflow: auto;
  padding: 0 16px 16px;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

th, td {
  border: 1px solid var(--border-subtle);
  padding: 10px 12px;
  text-align: left;
  color: var(--text-secondary);
}

th {
  background: rgba(255, 255, 255, 0.03);
  color: var(--text-primary);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  position: sticky;
  top: 0;
  z-index: 1;
}

tbody tr:hover td {
  background: rgba(255, 255, 255, 0.02);
}

.empty-table {
  color: var(--text-disabled);
  padding: 40px 16px;
  text-align: center;
  font-style: italic;
}

.type-tag {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
}

.type-tag.web {
  background: rgba(214, 186, 255, 0.15);
  color: var(--accent-primary);
}

.type-tag.back {
  background: rgba(173, 198, 255, 0.15);
  color: var(--accent-secondary);
}

.action-btns {
  display: flex;
  align-items: center;
  gap: 4px;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  transition: all 0.15s ease;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--text-primary);
}

.icon-btn.danger:hover {
  background: rgba(255, 100, 100, 0.15);
  color: #ff6464;
}

.icon-btn .material-icons {
  font-size: 16px;
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-content {
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: 12px;
  width: 480px;
  max-width: 90vw;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-default);
}

.modal-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--text-primary);
}

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.form-group input,
.form-group select {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  padding: 8px 12px;
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
}

.form-group input:focus,
.form-group select:focus {
  border-color: var(--accent-primary);
}

.form-group input::placeholder {
  color: var(--text-disabled);
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid var(--border-default);
}
</style>
