<script setup lang="ts">
import { computed, ref, watch } from 'vue'

const username = ref('')
const password = ref('')
const showPassword = ref(false)
const feedback = ref('')

const error = computed(() => username.value.includes(':') ? '用户名不能包含冒号。' : '')
const encoded = computed(() => {
  if (!username.value || error.value) return ''

  const bytes = new TextEncoder().encode(`${username.value}:${password.value}`)
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return btoa(binary)
})
const authorization = computed(() => encoded.value ? `Basic ${encoded.value}` : '')

watch([username, password], () => { feedback.value = '' })

async function copy(value: string) {
  try {
    await navigator.clipboard.writeText(value)
    feedback.value = '已复制到剪贴板。'
  } catch {
    feedback.value = '复制失败，请在结果框中手动复制。'
  }
}

function clear() {
  username.value = ''
  password.value = ''
  showPassword.value = false
  feedback.value = ''
}
</script>

<template>
  <div class="basic-auth">
    <header class="tool-header">
      <div>
        <h1>Basic 认证生成</h1>
        <p>生成 HTTP Authorization 请求头</p>
      </div>
      <button class="btn btn-ghost" type="button" @click="clear">
        <span class="material-icons">delete_outline</span>清空
      </button>
    </header>

    <div class="content">
      <section class="section">
        <h2><span class="material-icons">person_outline</span>认证信息</h2>
        <div class="fields">
          <label class="field">
            <span>用户名</span>
            <input v-model="username" type="text" autocomplete="off" placeholder="输入用户名" />
          </label>
          <label class="field">
            <span>密码</span>
            <span class="password-field">
              <input v-model="password" :type="showPassword ? 'text' : 'password'" autocomplete="new-password" placeholder="输入密码" />
              <button type="button" class="visibility-button" :title="showPassword ? '隐藏密码' : '显示密码'" :aria-label="showPassword ? '隐藏密码' : '显示密码'" @click="showPassword = !showPassword">
                <span class="material-icons">{{ showPassword ? 'visibility_off' : 'visibility' }}</span>
              </button>
            </span>
          </label>
        </div>
        <p v-if="error" class="error" role="alert">{{ error }}</p>
      </section>

      <section class="section results">
        <h2><span class="material-icons">code</span>生成结果</h2>
        <div class="result">
          <div class="result-header">
            <label for="authorization">Authorization 请求头值</label>
            <button class="btn btn-primary" type="button" :disabled="!authorization" @click="copy(authorization)">
              <span class="material-icons">content_copy</span>复制请求头值
            </button>
          </div>
          <textarea id="authorization" :value="authorization" readonly rows="3" placeholder="输入用户名后自动生成" />
        </div>
        <div class="result">
          <div class="result-header">
            <label for="encoded">Base64</label>
            <button class="btn btn-ghost" type="button" :disabled="!encoded" @click="copy(encoded)">
              <span class="material-icons">content_copy</span>复制 Base64
            </button>
          </div>
          <textarea id="encoded" :value="encoded" readonly rows="3" placeholder="输入用户名后自动生成" />
        </div>
        <p v-if="feedback" class="feedback" role="status">{{ feedback }}</p>
      </section>

      <p class="security-note"><span class="material-icons">info_outline</span>Base64 不是加密，请通过 HTTPS 传输并妥善保管认证信息。</p>
    </div>
  </div>
</template>

<style scoped>
.basic-auth { height: 100%; overflow: auto; padding: 24px; }
.tool-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 28px; }
.tool-header h1 { margin: 0 0 6px; font-size: 26px; line-height: 1.25; }
.tool-header p { margin: 0; color: var(--text-muted); font-size: 14px; }
.content { max-width: 900px; }
.section { padding: 20px 0 24px; border-top: 1px solid var(--border-default); }
h2 { display: flex; align-items: center; gap: 8px; margin: 0 0 18px; font-size: 15px; }
h2 .material-icons { font-size: 19px; color: var(--accent-primary); }
.fields { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px; }
.field { display: flex; flex-direction: column; gap: 8px; min-width: 0; color: var(--text-secondary); font-size: 13px; font-weight: 600; }
input, textarea { width: 100%; border: 1px solid var(--border-default); border-radius: 6px; background: rgba(0, 0, 0, 0.2); color: var(--text-primary); outline: none; font: 14px 'SF Mono', Monaco, monospace; }
input { height: 42px; padding: 0 12px; }
input:focus, textarea:focus { border-color: var(--accent-primary); }
.password-field { display: flex; position: relative; }
.password-field input { padding-right: 44px; }
.visibility-button { position: absolute; right: 2px; top: 2px; width: 38px; height: 38px; border: 0; background: transparent; color: var(--text-muted); cursor: pointer; }
.visibility-button:hover { color: var(--text-primary); }
.visibility-button .material-icons, .btn .material-icons { font-size: 18px; }
.results { display: flex; flex-direction: column; gap: 18px; }
.results h2 { margin-bottom: 0; }
.result-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 8px; }
.result-header label { color: var(--text-secondary); font-size: 13px; font-weight: 600; }
textarea { display: block; min-height: 76px; padding: 12px; line-height: 1.5; resize: vertical; overflow-wrap: anywhere; }
.error { margin: 12px 0 0; color: #ff8da1; font-size: 13px; }
.feedback { margin: 0; color: var(--text-secondary); font-size: 13px; }
.security-note { display: flex; align-items: flex-start; gap: 8px; margin: 0; color: var(--text-muted); font-size: 13px; line-height: 1.5; }
.security-note .material-icons { font-size: 18px; flex-shrink: 0; }
@media (max-width: 650px) {
  .basic-auth { padding: 20px 16px; }
  .fields { grid-template-columns: 1fr; }
  .result-header { flex-wrap: wrap; }
}
</style>
