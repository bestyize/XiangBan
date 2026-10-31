import { request } from './forum'
import type { Page } from './types'
import type { SchemeReference,SchemeInput } from './schemes'
export interface SchemeBackup {id:string;schema_version:1;title:string;summary:string;items:SchemeReference[];revision:number;status:'active'|'removed';updated_at:string}
export const schemeBackups={
  list:(page=1,status='active')=>request<Page<SchemeBackup>>('GET',`/me/scheme-backups?page=${page}&status=${status}`),
  save:(id:string,input:SchemeInput,expected_revision:number)=>request<{id:string;revision:number}>('PUT',`/me/scheme-backups/${encodeURIComponent(id)}`,{schema_version:1,...input,expected_revision}),
  remove:(row:SchemeBackup)=>request('POST',`/me/scheme-backups/${encodeURIComponent(row.id)}/remove`,{expected_revision:row.revision}),
}
