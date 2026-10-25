export interface TokenField {
  key: string;
  label: string;
  labelEn?: string;
  type: 'password' | 'text';
  required: boolean;
  placeholder?: string;
  description?: string;
}
export interface TokenSchema {
  title: string;
  titleEn?: string;
  description?: string;
  descriptionEn?: string;
  fields: TokenField[];
}
export interface InstalledConnector {
  /** Market slug; several MCP services can belong to the same plugin. */
  plugin?: string;
  id: string;
  name: string;
  nameEn?: string;
  description: string;
  version: string;
  source: 'market' | 'local';
  iconUrl: string | null;
  auth: { type: 'none' | 'token' | 'oauth' | 'env' | 'cli'; schema?: TokenSchema; configured: string[] };
  status: 'disconnected' | 'connecting' | 'connected' | 'needs_auth' | 'error' | 'disabled';
  enabled: boolean;
  error?: string;
  tools: { name: string; description: string }[];
  examples: string[];
  skills: string[];
  unsupported?: string;
}
export interface MarketConnector {
  displayName: { zh?: string; en?: string };
  displayDescription: { zh?: string; en?: string };
  examples: { zh?: string[]; en?: string[] };
  authTypes: InstalledConnector['auth']['type'][];
  tokenFields: { server: string; key: string; label: string; type: string; required: boolean }[];
  transports: string[];
  skills: string[];
  skillCount: number;
  disabledToolCount: number;
}
export interface ConnectorAuthorization {
  status?: string;
  connected?: boolean;
  flowId?: string;
  authorizationUrl?: string;
}
export const connectorStatus: Record<InstalledConnector['status'], string> = {
  disconnected: '未连接', connecting: '连接中', connected: '已连接', needs_auth: '需要授权', error: '连接失败', disabled: '已停用',
};
export const connectorAuth: Record<InstalledConnector['auth']['type'], string> = {
  none: '无需认证', token: '令牌认证', oauth: '网页登录授权', env: '环境变量', cli: '命令行登录',
};
