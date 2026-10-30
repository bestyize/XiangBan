export type BuiltinTool = 'computer' | 'phone' | 'design';

export const builtinToolLabels: Record<BuiltinTool, string> = {
  computer: '操作电脑', phone: '操作手机', design: '设计画板',
};

/** Carry the selection in the message, so queued messages and retries keep it too. */
export function withBuiltinTool(text: string, tool?: BuiltinTool | null): string {
  if (!tool) return text;
  const prefix: Record<BuiltinTool, string> = {
    computer: '请使用电脑控制工具完成以下任务：',
    phone: '请使用手机操作工具完成以下任务：',
    design: '请使用设计画板处理以下设计稿任务：',
  };
  return `${prefix[tool]}\n${text}`;
}
