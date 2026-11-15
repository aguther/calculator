// MIT; extracted behavioral reference from simple-calculator 0274b42.
  function isScalarTimeEntry(op){
    return mode==="time" && (op==="*" || op==="/") && entry!=="" && !entry.includes(":");
  }
  function entryValueForOp(op){
    if(mode==="time") return isScalarTimeEntry(op) ? core.numberEntryValue(entry) : core.entryToSeconds(entry);
    return core.numberEntryValue(entry);
  }
  function currentText(){
    if(error) return error;
    if(mode==="time"){
      return isScalarTimeEntry(opForNextValue()) ? core.fmtNum(core.numberEntryValue(entry)) : (entry==="" ? "0:00" : core.fmtTimeEntry(entry));
    }
    return entry==="" ? "0" : core.fmtNumEntry(entry);
  }
  function canCloseParen(){
    if(error || lastEntryWasResult || openParenCount()<=0) return false;
    if(entry!=="") return true;
    var last=lastExprStep();
    return isValueStep(last) || !!(last && last.type==="paren" && last.value===")");
  }
  function canEqual(){
    if(error || lastEntryWasResult || openParenCount()!==0) return false;
    if(entry!=="") return true;
    var last=lastExprStep();
    return isValueStep(last) || !!(last && last.type==="paren" && last.value===")");
  }
  function lastExprStep(){
    var relevant=expressionSteps();
    return relevant.length ? relevant[relevant.length-1] : null;
  }
  function isValueStep(step){ return step && step.type!=="sum" && step.type!=="paren"; }
  function canStartValue(){
    var last=lastExprStep();
    return !last || pendingOp || (last.type==="paren" && last.value==="(");
  }
  function openParenCount(){
    return expressionSteps().reduce(function(count,step){
      if(step.type==="paren" && step.value==="(") return count+1;
      if(step.type==="paren" && step.value===")") return count-1;
      return count;
    },0);
  }
  function opForNextValue(){
    var last=lastExprStep();
    if(!last) return pendingOp || null;
    if(last.type==="paren" && last.value==="(") return null;
    return pendingOp || "+";
  }
  function commitEntry(valueOverride){
    var op=opForNextValue();
    var scalar=isScalarTimeEntry(op);
    var value=valueOverride===undefined ? entryValueForOp(op) : valueOverride;
    steps.push({op:op,value:value,unit:scalar ? "scalar" : undefined});
    entry="";
    pendingOp=null;
  }
  function resetForNewExpression(){
    entry="";
    steps=[];
    pendingOp=null;
    lastEntryWasResult=false;
    error=null;
  }
  function pressDigit(digit){
    if(mode==="num" && !error && !lastEntryWasResult && entry.length>=64 && !(entry==="0" && digit!=="." && digit!==",")){
      return rejectInput("Maximal 64 Zeichen pro Eingabe. Bitte die letzte Stelle lÃ¶schen.");
    }
    if(error || lastEntryWasResult) resetForNewExpression();
    if(!canStartValue()) return false;
    if(mode==="time"){
      var unsigned=entry.charAt(0)==="-" ? entry.slice(1) : entry;
      var parts=unsigned.split(":");
      var colons=parts.length-1;
      if(colons>=1 && parts[colons].length>=2) return false;
      if(colons===0 && unsigned.length>=9) return false;
      entry+=digit;
    }else if(digit==="." || digit===","){
      if(entry.includes(".")) return false;
      if(entry==="") entry="0";
      entry+=".";
    }else{
      if(entry==="0") entry=digit;
      else entry+=digit;
    }
    completeChange();
    return true;
  }
  function pressOp(op){
    if(error) return false;
    if(lastEntryWasResult){
      entry="";
      lastEntryWasResult=false;
      pendingOp=op;
      completeChange();
      return true;
    }
    var before=lastExprStep();
    var additional=entry!=="" || !before || (before.type==="paren" && before.value==="(" && op==="-") ? 1 : 0;
    if(!hasHistoryRoom(additional)) return false;
    if(entry!=="") commitEntry();
    var last=lastExprStep();
    if(!last) commitEntry(0);
    else if(last.type==="paren" && last.value==="("){
      if(op==="-") commitEntry(0);
      else return false;
    }else if(!isValueStep(last) && !(last.type==="paren" && last.value===")")){
      return false;
    }
    pendingOp=op;
    completeChange();
    return true;
  }
  function pressEquals(){
    if(!canEqual()) return false;
    if(!hasHistoryRoom((entry!=="" ? 1 : 0)+1)) return false;
    if(entry!=="") commitEntry();
    var result=evaluate();
    if(!Number.isFinite(result)){
      error="Nicht definiert";
      entry="";
      pendingOp=null;
      lastEntryWasResult=false;
      completeChange();
      return true;
    }
    steps.push({type:"sum",value:result});
    entry=core.valueToEntry(result,mode);
    pendingOp=null;
    lastEntryWasResult=true;
    completeChange();
    return true;
  }
  function pressColonTime(){
    if(error || lastEntryWasResult) resetForNewExpression();
    if(!canStartValue()) return false;
    if((entry.match(/:/g) || []).length>=2) return false;
    entry+=":";
    completeChange();
    return true;
  }
  function pressOpenParen(){
    if(error) return false;
    if(!hasHistoryRoom((!lastEntryWasResult && entry!=="" ? 1 : 0)+1,lastEntryWasResult)) return false;
    if(lastEntryWasResult) resetForNewExpression();
    if(entry!=="") commitEntry();
    var last=lastExprStep();
    var op=null;
    if(last && (isValueStep(last) || (last.type==="paren" && last.value===")"))) op=pendingOp || "*";
    else op=pendingOp;
    steps.push({type:"paren",value:"(",op:op});
    pendingOp=null;
    completeChange();
    return true;
  }
  function pressCloseParen(){
    if(!canCloseParen()) return false;
    if(!hasHistoryRoom((entry!=="" ? 1 : 0)+1)) return false;
    if(entry!=="") commitEntry();
    steps.push({type:"paren",value:")"});
    pendingOp=null;
    completeChange();
    return true;
  }
  function backspace(){
    if(error){ resetForNewExpression(); completeChange(); return true; }
    if(entry!=="") entry=entry.slice(0,-1);
    else if(pendingOp) pendingOp=null;
    else if(steps.length>0) steps.pop();
    else return false;
    completeChange();
    return true;
  }
  function clearAll(){
    resetForNewExpression();
    completeChange();
    return true;
  }
  function clearEntry(){
    if(error || lastEntryWasResult) return clearAll();
    if(entry==="") return false;
    entry="";
    completeChange();
    return true;
  }
  function undo(){
    if(error){ resetForNewExpression(); completeChange(); return true; }
    if(lastEntryWasResult){
      lastEntryWasResult=false;
      entry="";
      pendingOp=null;
      if(steps.length>0 && steps[steps.length-1].type==="sum") steps.pop();
      completeChange();
      return true;
    }
    if(entry!=="") entry="";
    else if(pendingOp) pendingOp=null;
    else if(steps.length>0){ steps.pop(); pendingOp=null; }
    else return false;
    completeChange();
    return true;
  }