// Original repository V8JavaScriptRuntime, public host contract only.
#include "javascript/v8_javascript_runtime.h"
#include <array>
#include <iostream>
#include <string>
using namespace layoutng;
struct Host final:JavaScriptHostBindings {
 static constexpr std::array<std::string_view,3> names={"nativeGetter","nativeObject","capture"};
 JavaScriptFunction captured;
 std::span<const std::string_view> GlobalNames()const override{return names;}
 HostResult Invoke(const HostCall& call)override{
  if(call.operation==HostOperation::kGet&&call.receiver==0&&call.member=="nativeGetter")return HostResult::Failure(JavaScriptExceptionKind::kTypeError,"native getter failure");
  if(call.operation==HostOperation::kGet&&call.receiver==0&&call.member=="nativeObject")return {.value=HostObjectRef{1}};
  if(call.operation==HostOperation::kGet&&call.receiver==1&&call.member=="valueOf")return {.value=HostMethodRef{1,"valueOf"}};
  if(call.operation==HostOperation::kCall&&call.receiver==1&&call.member=="valueOf")return HostResult::Failure(JavaScriptExceptionKind::kTypeError,"native coercion failure");
  if(call.operation==HostOperation::kGet&&call.receiver==0&&call.member=="capture")return {.value=HostMethodRef{0,"capture"}};
  if(call.operation==HostOperation::kCall&&call.receiver==0&&call.member=="capture"){captured=std::get<JavaScriptFunction>(call.arguments[0]);return {};}
  return {.handled=false};
 }
};
void emit(std::string_view name,const JavaScriptException& e){std::cout<<name<<'\t'<<e.source_name<<'\t'<<e.line<<'\t'<<e.column<<'\t'<<e.source_line<<'\t'<<e.message<<'\n';}
int main(){Host host;V8JavaScriptRuntime runtime;auto realm=runtime.CreateRealm(host);
 const std::array<std::pair<const char*,const char*>,16> cases={{
 {"primitive","\nthrow 'primitive';"},
 {"finally-caught","try {\n throw 'original';\n} finally {\n try { throw 'cleanup'; } catch(e) {}\n}"},
 {"finally-overrides","try {\n throw 'original';\n} finally {\n throw 'replacement';\n}"},
 {"nested-finally","try {\n try { throw 'nested'; } finally {var a=1;}\n} finally {var b=2;}"},
 {"caught-same","try {throw 'same'} catch(e) {}\nthrow 'same';"},
 {"crlf","var x=1;\r\n throw 'crlf';"},
 {"unicode","var s='😀'; throw 'unicode';"},
 {"separator","var x=1;  throw 'separator';"},

 {"old-error","var old = new Error('old');\n\nthrow old;"},
 {"caught-new","try { throw 'caught'; } catch(e) {}\nthrow 'new';"},
 {"rethrow","try {\n throw 'caught';\n} catch(e) {\n throw e;\n}"},
 {"finally","try {\n throw 'original';\n} finally {\n var cleanup=1;\n}"},
 {"iterator-cleanup","for(const x of {[Symbol.iterator](){return {next(){return {value:1,done:false}},return(){throw 'cleanup'}}}}){\n throw 'original';\n}"},
 {"getter","\nnativeGetter;"},
 {"coercion","\n+nativeObject;"},
 {"nested","function inner(){\n throw 'inner';\n}\nfunction outer(){inner()}\nouter();"},
 }};
 for(auto [name,source]:cases){auto result=runtime.Evaluate(realm,source,std::string("https://test/")+name+".js");if(result.exception)emit(name,*result.exception);else std::cout<<name<<"\tSUCCESS\n";}
 auto result=runtime.Evaluate(realm,"capture(function invoked(){\n throw 'called';\n});","https://test/call.js");result=runtime.Call(realm,host.captured,JavaScriptUndefined{},{});if(result.exception)emit("call",*result.exception);
 runtime.Evaluate(realm,"capture(function queued(){\n throw 'microtask';\n});","https://test/microtask.js");runtime.EnqueueMicrotask(realm,host.captured);runtime.PerformMicrotaskCheckpoint();for(auto& e:runtime.TakePendingExceptions(realm))emit("microtask",e);
 runtime.Evaluate(realm,"capture(function queuedError(){\n throw new Error('microtask error');\n});","https://test/microtask-error.js");runtime.EnqueueMicrotask(realm,host.captured);runtime.PerformMicrotaskCheckpoint();for(auto& e:runtime.TakePendingExceptions(realm))emit("microtask-error",e);
 runtime.Evaluate(realm,"capture(function queuedOld(){\n throw old;\n});","https://test/microtask-old.js");runtime.EnqueueMicrotask(realm,host.captured);runtime.PerformMicrotaskCheckpoint();for(auto& e:runtime.TakePendingExceptions(realm))emit("microtask-old",e);
 runtime.Evaluate(realm,"var unthrown = new Error('unthrown');","https://test/unthrown.js");
 runtime.Evaluate(realm,"capture(function queuedUnthrown(){\n throw unthrown;\n});","https://test/microtask-unthrown.js");runtime.EnqueueMicrotask(realm,host.captured);runtime.PerformMicrotaskCheckpoint();for(auto& e:runtime.TakePendingExceptions(realm))emit("microtask-unthrown",e);
 runtime.Evaluate(realm,"Promise.reject(old);var readStack=new Error('read stack');readStack.stack;Promise.reject(readStack);","https://test/rejection-read.js");runtime.PerformMicrotaskCheckpoint();for(auto& e:runtime.TakePendingExceptions(realm))emit("rejection-read",e);
 runtime.Evaluate(realm,"var rejectionOld=new Error('rejected old');\nPromise.reject(rejectionOld);\nPromise.reject('primitive rejection');","https://test/rejection.js");runtime.PerformMicrotaskCheckpoint();for(auto& e:runtime.TakePendingExceptions(realm))emit("rejection",e);
}
