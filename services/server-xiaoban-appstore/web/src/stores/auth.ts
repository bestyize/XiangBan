import { defineStore } from 'pinia';
import { ref } from 'vue';
import { api } from '../api';

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('xiaoban_appstore_admin_token'));
  const username = ref<string | null>(localStorage.getItem('xiaoban_appstore_admin_user'));
  const isAuthenticated = ref<boolean>(!!token.value);

  async function login(cred: { username: string; password: string }) {
    const res = await api.login(cred);
    token.value = res.token;
    username.value = res.username;
    isAuthenticated.value = true;
    localStorage.setItem('xiaoban_appstore_admin_token', res.token);
    localStorage.setItem('xiaoban_appstore_admin_user', res.username);
    return res;
  }

  function logout() {
    token.value = null;
    username.value = null;
    isAuthenticated.value = false;
    localStorage.removeItem('xiaoban_appstore_admin_token');
    localStorage.removeItem('xiaoban_appstore_admin_user');
  }

  function updateCredentials(newUsername: string, newToken: string) {
    username.value = newUsername;
    token.value = newToken;
    isAuthenticated.value = true;
    localStorage.setItem('xiaoban_appstore_admin_token', newToken);
    localStorage.setItem('xiaoban_appstore_admin_user', newUsername);
  }

  return {
    token,
    username,
    isAuthenticated,
    login,
    logout,
    updateCredentials,
  };
});
