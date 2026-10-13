export interface ModelFailure { message: string; refresh: boolean }
/** Only explicit provider error content identifies a retired model; HTTP 404 alone does not. */
export function modelFailure(raw: string): ModelFailure {
  const value = raw.toLowerCase();
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
