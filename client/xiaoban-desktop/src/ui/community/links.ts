import { invoke } from '../../bridge.ts';

export const FORUM_URL = 'https://chat.xiangban.xyz/';
export const CONTACT_URL = 'https://chat.xiangban.xyz/contact';

export async function openInBrowser(url: string, desktop: boolean) {
  if (desktop) await invoke('open_download_url', { url });
  else window.open(url, '_blank', 'noopener,noreferrer');
}
