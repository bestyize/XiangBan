<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { editorApi, type Author, type Token } from '@/api/editor'
import { fmtTime } from './shared'
const authors = ref<Author[]>([]), tokens = ref<Token[]>([]), enabled = ref(false), busy = ref(false), error = ref(''), message = ref(''), oneTimeToken = ref('')
const creating = ref(false), binding = ref('')
const form = reactive({ request_id: crypto.randomUUID(), username: '', password: '', nickname: '', avatar_url: '', trust_level: 1 })
const tokenForm = reactive({ name: 'Claude Code', author_uid: '', can_publish: false, days: 90 })
async function load() { const [a, t] = await Promise.all([editorApi.authors(), editorApi.tokens()]); authors.value = a.items; enabled.value = a.provision_enabled; tokens.value = t.items; if (!tokenForm.author_uid) tokenForm.author_uid = a.items[0]?.uid || '' }
async function act(fn: () => Promise<unknown>, success: string) { busy.value = true; error.value = ''; message.value = ''; try { await fn(); message.value = success; await load() } catch (e) { error.value = e instanceof Error ? e.message : String(e) } finally { busy.value = false } }
async function bindExisting() { await act(() => editorApi.bindAuthor(binding.value.trim()), '已绑定') }
async function revoke(id: string) { await act(() => editorApi.revoke(id), '令牌已撤销') }
async function create() { await act(async () => { await editorApi.createAuthor(form); form.password = ''; form.request_id = crypto.randomUUID(); creating.value = false }, '编辑账号已创建') }
async function makeToken() { oneTimeToken.value = ''; await act(async () => { const r = await editorApi.createToken(tokenForm); oneTimeToken.value = r.token }, '令牌已生成，请复制保存') }
async function copyToken() { try { await navigator.clipboard.writeText(oneTimeToken.value); message.value = '已复制' } catch { error.value = '自动复制失败，请手动选中令牌复制' } }
onMounted(() => act(load, ''))
</script>
<template>
  <p class="muted">编辑账号用于发布站方内容。可创建新账号，也可绑定已有论坛用户；已有用户需至少为成员。</p>
  <p v-if="error" class="error-text" role="alert">{{ error }}</p><p v-if="message" role="status">{{ message }}</p>
  <div class="adm-toolbar"><button class="btn btn-primary" :disabled="busy || !enabled" @click="creating = !creating">创建编辑账号</button><RouterLink class="btn" to="/admin/editor">批量发帖</RouterLink></div>
  <p v-if="!enabled" class="muted">账号创建通道尚未配置，可先在统一账号中心创建用户并登录论坛，再绑定为编辑账号。配置方式见仓库论坛 README。</p>
  <form v-if="creating" class="panel editor-form" @submit.prevent="create">
    <h2>创建编辑账号</h2>
    <label>登录用户名<input v-model="form.username" class="input" required maxlength="20" pattern="[A-Za-z一-鿿][A-Za-z0-9_一-鿿]*" autocomplete="off" placeholder="例如 ForumEditor" /></label>
    <label>初始密码<input v-model="form.password" class="input" type="password" required minlength="8" maxlength="128" autocomplete="new-password" /></label>
    <label>论坛展示名<input v-model="form.nickname" class="input" required maxlength="24" placeholder="例如 超级小伴编辑部" /></label>
    <label>头像地址（选填）<input v-model="form.avatar_url" class="input" type="url" placeholder="https://…" /></label>
    <label>信任等级<select v-model.number="form.trust_level" class="select"><option :value="1">成员</option><option :value="2">老用户</option><option :value="3">守护者（可发官方板块）</option></select></label>
    <p class="muted">账号可用用户名和密码登录，邮箱、手机号可由用户登录后自行绑定。</p>
    <button class="btn btn-primary" :disabled="busy">{{ busy ? '创建中…' : '创建账号' }}</button>
  </form>
  <form class="adm-toolbar" @submit.prevent="bindExisting"><input v-model="binding" class="input" placeholder="已有论坛用户的完整 UID" required /><button class="btn" :disabled="busy">绑定已有用户</button></form>
  <div class="panel adm-card"><table class="adm-table"><thead><tr><th>展示名</th><th>UID</th><th>等级 / 状态</th></tr></thead><tbody><tr v-for="a in authors" :key="a.uid"><td>{{ a.nickname }}</td><td class="adm-mono">{{ a.uid }}</td><td>{{ a.trust_level }} / {{ a.status }}</td></tr></tbody></table><p v-if="!authors.length" class="adm-empty">尚无编辑账号</p></div>
  <h2>本地创作令牌</h2><p class="muted">每个令牌固定绑定一个作者，可以随时撤销。默认只允许导入待发布稿。</p>
  <form class="panel editor-form" @submit.prevent="makeToken">
    <label>名称<input v-model="tokenForm.name" class="input" required maxlength="60" /></label>
    <label>发帖作者<select v-model="tokenForm.author_uid" class="select" required><option value="" disabled>请选择</option><option v-for="a in authors.filter(a => a.status === 'active')" :key="a.uid" :value="a.uid">{{ a.nickname }}</option></select></label>
    <label>有效天数<input v-model.number="tokenForm.days" class="input" type="number" min="1" max="365" required /></label>
    <label><input v-model="tokenForm.can_publish" type="checkbox" /> 允许本地直接发布</label>
    <button class="btn" :disabled="busy || !authors.length">生成令牌</button>
  </form>
  <div v-if="oneTimeToken" class="panel editor-form"><strong>令牌仅在创建时显示，请妥善保存</strong><textarea :value="oneTimeToken" class="input token-output" readonly aria-label="新建的创作令牌" /><div><button class="btn" @click="copyToken">复制令牌</button><button class="btn btn-ghost" @click="oneTimeToken = ''">关闭显示</button></div></div>
  <div class="panel adm-card"><table class="adm-table"><thead><tr><th>名称</th><th>权限</th><th>有效期</th><th>操作</th></tr></thead><tbody><tr v-for="t in tokens" :key="t.id"><td>{{ t.name }}<div class="adm-sub">{{ authors.find(a => a.uid === t.author_uid)?.nickname || t.author_uid }}</div></td><td>{{ t.can_publish ? '导入、发布' : '仅导入' }}</td><td>{{ t.revoked_at ? '已撤销' : fmtTime(t.expires_at) }}</td><td><button class="btn btn-sm" :disabled="busy || !!t.revoked_at" @click="revoke(t.id)">撤销</button></td></tr></tbody></table></div>
</template>
<style scoped>
.editor-form{display:grid;gap:14px;padding:20px;margin:16px 0;max-width:700px}.editor-form label{display:grid;gap:6px}.editor-form label:has(input[type=checkbox]){display:block}.token-output{min-height:110px;resize:vertical;word-break:break-all}h2{font-size:18px;margin:24px 0 10px}.adm-toolbar{flex-wrap:wrap}.adm-mono{word-break:break-all}
</style>
