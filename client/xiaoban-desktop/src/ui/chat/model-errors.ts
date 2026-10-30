export interface ModelFailure { message: string; refresh: boolean }
/** Only explicit provider error content identifies a retired model; HTTP 404 alone does not. */
export function modelFailure(raw: string): ModelFailure {
  const value = raw.toLowerCase();
  if (/request[_ -]too[_ -]large|\b413\b/.test(value)) {
    return { message: '会话内容太大，无法发送。请减少图片或新开会话后继续。', refresh: false };
  }
  if (/model[_ -](not[_ -]found|not[_ -]supported|deprecated|disabled)|model.*(does not exist|no longer available|has been retired)|模型.*(不存在|已停用|已下线)/i.test(raw)) {
    return { message: '当前模型已不可用，可以切换模型后继续。', refresh: true };
  }
  if (/invalid[_ -]api[_ -]key|authentication|unauthorized|认证失败|凭证.*失效|\b401\b/.test(value)) {
    return { message: '当前服务认证失败，请修改配置或切换模型。', refresh: true };
  }
  if (/insufficient[_ -]quota|insufficient[_ -]balance|余额不足|额度.*(不足|耗尽)|quota.*exceed/.test(value)) {
    return { message: '当前服务额度不足，可以切换模型。', refresh: false };
  }
  if (/permission|forbidden|无.*权限|\b403\b/.test(value)) {
    return { message: '当前账号没有使用权限，请检查配置或切换模型。', refresh: false };
  }
  if (/rate[_ -]limit|\b429\b|限流|繁忙/.test(value)) {
    return { message: '当前服务暂时繁忙，可以稍后重试或切换模型。', refresh: false };
  }
  if (/\b404\b/.test(value)) return { message: '当前服务配置可能有误，请检查地址和协议或切换模型。', refresh: false };
  return { message: '暂时无法完成回复，可以重试或切换模型。', refresh: false };
}

/** The reason shown while the host retries a model request by itself; `message` is the provider's own wording. */
export function retryReason(message: string): string {
  if (/[\u4e00-\u9fa5]/.test(message)) return message.replace(/[，。,.；;]?正在重试.*$/, '').replace(/[。.]$/, '');
  const value = message.toLowerCase();
  if (/response headers|first model event|stream data|timed out|timeout/.test(value)) return '模型服务长时间没有响应';
  if (/\b429\b|rate[_ -]limit/.test(value)) return '模型服务繁忙，请求被限流';
  if (/\b(408|425|5\d\d)\b|overloaded/.test(value)) return '模型服务暂时出错';
  if (/stream ended|reset|closed|connect|request failed|stream read/.test(value)) return '与模型的连接中断了';
  return '与模型的连接不稳定';
}

/** The status line for an automatic retry: what went wrong, and that the host is already trying again. */
export function retryNotice(attempt: number, message: string): string {
  return `${retryReason(message)}，正在进行第 ${attempt} 次重试…`;
}

/** "12 秒", "1 分 05 秒". */
export function waitedFor(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  return s < 60 ? `${s} 秒` : `${Math.floor(s / 60)} 分 ${String(s % 60).padStart(2, '0')} 秒`;
}

/** Seconds of silence from the model after which the page says so instead of a bare "思考中…". */
export const SILENCE_NOTICE_SECONDS = 10;

/** The status while a model request has been silent for a while; `retryStatus` is the retry line, when one is showing. */
export function silenceNotice(seconds: number, retryStatus = ''): string {
  if (seconds < SILENCE_NOTICE_SECONDS) return retryStatus;
  return retryStatus
    ? `${retryStatus}（已等待 ${waitedFor(seconds)}）`
    : `模型暂时没有新输出，已等待 ${waitedFor(seconds)}；长时间没有响应会自动重试。`;
}

/** "1.2 KB", "32 KB". */
export function kilobytes(bytes: number): string {
  const kb = bytes / 1024;
  return kb < 10 ? `${kb.toFixed(1)} KB` : `${Math.round(kb)} KB`;
}

/** The status while the model writes out a tool call's arguments, which can take minutes for a big file. */
export function toolInputNotice(name: string, path: string | null | undefined, bytes: number): string {
  const target = path ? ` ${path}` : '';
  return name === 'write_file' || name === 'edit_file'
    ? `正在写入${target}，已收到 ${kilobytes(bytes)}…`
    : `正在生成工具调用${name ? `「${name}」` : ''}的内容${target}，已收到 ${kilobytes(bytes)}…`;
}
