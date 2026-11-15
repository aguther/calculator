const fs = require('node:fs');
const core = require('../tests/reference/calculator-core.js');
const fixtures = [];
for (const entry of ['', '0','45','145','1230','1:2','1::15','-145','999999999','99:99:99']) {
 fixtures.push({kind:'parse',input:entry,expected:core.entryToSeconds(entry)});
}
for (const n of [0,-0.5,-1.5,0.5,59.5,-2700,3600,3615,1.2345675,-1.2345675,999999999999,1e21,1e-7]) {
 fixtures.push({kind:'time',input:n,expected:core.fmtSeconds(n)});
 fixtures.push({kind:'number',input:n,expected:core.fmtNum(n)});
}
let seed=42;
for(let i=0;i<300;i++) {
 seed=(Math.imul(seed,1664525)+1013904223)>>>0;
 const a=seed%10000,b=(seed>>>8)%100+1,c=(seed>>>16)%100+1;
 const op=['+','-','*','/'][seed%4];
 const steps=[{value:a,op:null},{value:b,op},{value:c,op:'*'}];
 fixtures.push({kind:'evaluate',input:steps,expected:core.evaluateSteps(steps)});
}
const text=JSON.stringify(fixtures,null,2)+'\n';
const path='tests/reference/fixtures.json';
if(process.argv.includes('--check')) {if(fs.readFileSync(path,'utf8')!==text)throw new Error('Reference fixtures differ');}
else fs.writeFileSync(path,text);

const vm=require('node:vm');
const functions=fs.readFileSync('tests/reference/state-machine.js','utf8');
const sequences=['45-130=','1:30*2=','(2+3)*4=','2+3=*4=','1/0=7','2+3=u','2(3+4)=','1:2:3+45=','2+3==','12+b3=','99','199'];
let random=987;
for(let i=0;i<200;i++){
 let sequence='';for(let j=0;j<24;j++){random=(Math.imul(random,1664525)+1013904223)>>>0;sequence+='0123456789+-*/()=bucC:'[random%21];}sequences.push(sequence);
}
const states=[];
for(const mode of ['time','num'])for(const keys of sequences){
 const context={core};vm.createContext(context);
 const setup=`var mode=${JSON.stringify(mode)},entry='',steps=[],pendingOp=null,lastEntryWasResult=false,error=null;
 function expressionSteps(){return core.expressionSteps(steps);}; function completeChange(){}; function evaluate(){return core.evaluateSteps(steps);}
 function rejectInput(){return false;};function hasHistoryRoom(additional,fresh){return (fresh?0:steps.length)+additional<=1000;}`;
 vm.runInContext(setup+'\n'+functions,context);
 for(const key of keys){context.key=key;vm.runInContext(`if(key>='0'&&key<='9')pressDigit(key);else if('+-*/'.includes(key))pressOp(key);else if(key==='=')pressEquals();else if(key==='(')pressOpenParen();else if(key===')')pressCloseParen();else if(key===':')mode==='num'?pressDigit('.'):pressColonTime();else if(key==='b')backspace();else if(key==='u')undo();else if(key==='c')clearEntry();else if(key==='C')clearAll();`,context);}
 const snapshot=vm.runInContext('({current:currentText(),total:evaluate(),rows:steps.length,pending:pendingOp,result:lastEntryWasResult,error:!!error})',context);
 states.push({mode,keys,...snapshot});
}
const stateText=JSON.stringify(states,null,2)+'\n';const statePath='tests/reference/states.json';
if(process.argv.includes('--check')){if(fs.readFileSync(statePath,'utf8')!==stateText)throw new Error('State fixtures differ');}else fs.writeFileSync(statePath,stateText);
