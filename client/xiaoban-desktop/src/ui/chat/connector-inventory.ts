import { invoke } from '../../bridge.ts';
import { connectorStatus, type InstalledConnector } from '../ai-store/connector-plugins.ts';

export interface ConnectorInventoryEntry {
  id: string; name: string; connected: boolean; enabled: boolean; toolCount: number;
  detail?: string; iconUrl?: string | null;
}
/** Merge installed plugins (including unconfigured ones) with manually configured MCPs. */
export async function loadConnectorInventory(): Promise<{ items: ConnectorInventoryEntry[]; warning: string }> {
  async function list<T>(command: string): Promise<T[]> {
    const result = await invoke<T[]>(command);
    if (!Array.isArray(result)) throw new Error('连接器列表格式无效');
    return result;
  }
  const [plugins, legacy] = await Promise.allSettled([
    list<InstalledConnector>('ai_connector_installed'), list<ConnectorInventoryEntry>('ai_store_connector_status'),
  ]);
  if (plugins.status === 'rejected' && legacy.status === 'rejected') throw new Error('读取连接器失败，请重试');
  const items = new Map<string, ConnectorInventoryEntry>();
  if (legacy.status === 'fulfilled') for (const item of legacy.value) items.set(item.id, item);
  if (plugins.status === 'fulfilled') for (const item of plugins.value) items.set(item.id, {
    id: item.id, name: item.name, enabled: item.enabled, connected: item.status === 'connected' && !item.unsupported,
    toolCount: item.tools.length, iconUrl: item.iconUrl,
    detail: item.unsupported ? `暂不支持：${item.unsupported}` : item.status === 'connected' ? undefined : connectorStatus[item.status],
  });
  return { items: [...items.values()], warning: plugins.status === 'rejected' || legacy.status === 'rejected' ? '部分连接器未能读取，请重试' : '' };
}
