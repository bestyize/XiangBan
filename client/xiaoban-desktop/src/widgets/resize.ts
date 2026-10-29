/** Serialize begin/move/end so a quick release cannot overtake the first native call. */
export function cornerResize(send:(phase:'begin'|'move'|'end'|'cancel')=>Promise<unknown>,failed:(error:unknown)=>void){
 let active=false,queued=false,ending:'end'|'cancel'|undefined,busy=false;
 async function pump(){
  if(busy)return;busy=true;
  try{
   while(queued||ending){
    if(ending){const phase=ending;ending=undefined;queued=false;active=false;await send(phase);}
    else{queued=false;await send('move');}
   }
  }catch(e){active=false;queued=false;ending=undefined;try{await send('cancel');}catch{}failed(e);}
  finally{busy=false;}
 }
 return {
  async begin(){if(active)return;active=true;busy=true;try{await send('begin');}catch(e){active=false;queued=false;ending=undefined;failed(e);}finally{busy=false;if(active)void pump();}},
  move(){if(active){queued=true;void pump();}},
  finish(cancel=false){if(active){ending=cancel?'cancel':'end';void pump();}},
  get active(){return active;},
 };
}
