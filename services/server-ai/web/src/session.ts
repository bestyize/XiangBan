import { reactive } from 'vue';

const TOKEN_KEY = 'ai_admin_token';
const NAME_KEY = 'ai_admin_name';

export const session = reactive({
  token: sessionStorage.getItem(TOKEN_KEY) ?? '',
  username: sessionStorage.getItem(NAME_KEY) ?? '',
});

export function signIn(token: string, username: string) {
  session.token = token;
  session.username = username;
  sessionStorage.setItem(TOKEN_KEY, token);
  sessionStorage.setItem(NAME_KEY, username);
}

export function signOut() {
  session.token = '';
  session.username = '';
  sessionStorage.removeItem(TOKEN_KEY);
  sessionStorage.removeItem(NAME_KEY);
}
