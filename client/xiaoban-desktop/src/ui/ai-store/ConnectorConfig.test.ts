// @vitest-environment happy-dom
import {mount} from '@vue/test-utils';
import {describe,it,expect} from 'vitest';
import ConnectorConfig from './ConnectorConfig.vue';
const output=(w:ReturnType<typeof mount>)=>JSON.parse(String(w.emitted('update:modelValue')!.at(-1)![0]));
describe('connector configuration forms',()=>{
 it('defaults public HTTP services to automatic authentication with JSON collapsed',()=>{
  const w=mount(ConnectorConfig,{props:{modelValue:JSON.stringify({url:'https://service.test/mcp'})}});
  expect(output(w).authMode).toBe('auto');expect(w.get('details.advanced-config').attributes('open')).toBeUndefined();expect(w.emitted('valid')!.at(-1)![0]).toBe(true);w.unmount();
 });
 it('collects environment credentials without asking the user to edit JSON',async()=>{
  const w=mount(ConnectorConfig,{props:{modelValue:JSON.stringify({command:'npx',args:['server'],env:{API_KEY:'${API_KEY}'}})}});
  expect(w.emitted('valid')!.at(-1)![0]).toBe(false);expect(w.get('input[aria-label="API_KEY"]').attributes('type')).toBe('password');
  await w.get('input[aria-label="API_KEY"]').setValue('a"b\\c');expect(output(w).env.API_KEY).toBe('a"b\\c');expect(w.emitted('valid')!.at(-1)![0]).toBe(true);w.unmount();
 });
 it('substitutes URL placeholders while preserving the service URL and request shape',async()=>{
  const w=mount(ConnectorConfig,{props:{modelValue:JSON.stringify({url:'https://service.test/mcp?key=YOUR_API_KEY'})}});
  await w.get('input[aria-label="API_KEY"]').setValue('secret-value');expect(output(w).url).toBe('https://service.test/mcp?key=secret-value');expect(output(w).headers).toBeUndefined();w.unmount();
 });
 it('removes a manual Authorization header when switching to browser authorization',async()=>{
  const w=mount(ConnectorConfig,{props:{modelValue:JSON.stringify({url:'https://service.test/mcp',headers:{Authorization:'Bearer ${TOKEN}'}})}});
  await w.get('select[aria-label="认证方式"]').setValue('oauth');expect(output(w).headers.Authorization).toBeUndefined();expect(output(w).authMode).toBe('oauth');expect(w.find('input[aria-label="TOKEN"]').exists()).toBe(false);w.unmount();
 });
 it('requires a key in Token mode and flags malformed advanced configuration',async()=>{
  const w=mount(ConnectorConfig,{props:{modelValue:JSON.stringify({url:'https://service.test/mcp'})}});
  await w.get('select').setValue('token');expect(w.emitted('valid')!.at(-1)![0]).toBe(false);await w.get('input[aria-label="API Key / Token"]').setValue('my-token');expect(output(w).headers.Authorization).toBe('Bearer my-token');
  await w.setProps({modelValue:'{'});expect(w.get('[role="alert"]').text()).toContain('JSON');expect(w.emitted('valid')!.at(-1)![0]).toBe(false);w.unmount();
 });
 it('keeps tenant routing headers while allowing a separate bearer key',async()=>{
  const w=mount(ConnectorConfig,{props:{modelValue:JSON.stringify({url:'https://service.test/mcp',headers:{'X-Tenant':'team-a'}})}});
  expect(output(w).authMode).toBe('auto');
  await w.get('select').setValue('token');await w.get('input[aria-label="API Key / Token"]').setValue('tenant-token');
  expect(output(w).headers).toEqual({'X-Tenant':'team-a',Authorization:'Bearer tenant-token'});w.unmount();
 });
});
