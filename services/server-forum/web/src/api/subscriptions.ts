import { request } from './forum'
import { subscriptionInput } from '../lib/subscriptionDraft'
export type Schedule = {kind:'daily';time:string}|{kind:'weekly';time:string;weekday:number}|{kind:'once';at:string}
export interface Subscription {id:string;revision:number;title:string;interests:string;recipient:string;schedule:Schedule;timezone:string;budget_per_run:number|null;budget_per_month:number|null;next_run:string|null;status:string;verified_at:string|null}
export interface SubscriptionInput {id:string;revision:number;title:string;interests:string;recipient:string;schedule:Schedule;timezone:string;budget_per_run:number|null;budget_per_month:number|null;active:boolean;confirmed:boolean}
export interface EditionCursor {created_at:string;id:string}
export interface Edition {id:string;subscription_id:string;title:string;status:string;error:string|null;accounting_status?:string;created_at:string;sent_at:string|null;due_at:string;plaintext?:string;html?:string;sources?:{title:string;url:string}[]}
export const subscriptions = {
 list:()=>request<{items:Subscription[];delivery_configured:boolean;worker_enabled:boolean}>('GET','/me/information-subscriptions'),
 save:(input:SubscriptionInput)=>request<Subscription>(input.revision?'PUT':'POST',`/me/information-subscriptions${input.revision?'/'+input.id:''}`,subscriptionInput(input,input.active,input.confirmed)),
 remove:(s:Subscription)=>request('DELETE',`/me/information-subscriptions/${s.id}`,{revision:s.revision}),
 verification:(id:string)=>request('POST',`/me/information-subscriptions/${id}/verification`),
 verify:(id:string,code:string)=>request('POST',`/me/information-subscriptions/${id}/verify`,{code}),
 history:(cursor?:EditionCursor)=>request<{items:Edition[];next_cursor:EditionCursor|null}>('GET','/me/information-editions'+(cursor?'?'+new URLSearchParams({before:cursor.created_at,before_id:cursor.id}).toString():'')),
 edition:(id:string)=>request<Edition>('GET',`/me/information-editions/${id}`),
}
