(function() {var implementors = {};
implementors["actix_optional_middleware"] = [{"text":"impl&lt;S&gt; Transform&lt;S, ServiceRequest&gt; for <a class=\"struct\" href=\"actix_optional_middleware/struct.Dummy.html\" title=\"struct actix_optional_middleware::Dummy\">Dummy</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Service&lt;ServiceRequest, Response = ServiceResponse&lt;AnyBody&gt;, Error = Error&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;S::Future: 'static,&nbsp;</span>","synthetic":false,"types":["actix_optional_middleware::Dummy"]},{"text":"impl&lt;D, R, S, DS, RS&gt; Transform&lt;S, ServiceRequest&gt; for <a class=\"enum\" href=\"actix_optional_middleware/enum.Group.html\" title=\"enum actix_optional_middleware::Group\">Group</a>&lt;D, R, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Service&lt;ServiceRequest, Response = ServiceResponse&lt;AnyBody&gt;, Error = Error&gt; + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;D: Transform&lt;S, ServiceRequest, Transform = DS, InitError = <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>, Error = Error&gt; + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;R: Transform&lt;S, ServiceRequest, Transform = RS, InitError = <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>, Error = Error&gt; + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;DS: Service&lt;ServiceRequest, Error = Error, Response = ServiceResponse&gt; + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;RS: Service&lt;ServiceRequest, Error = Error, Response = ServiceResponse&gt; + 'static,&nbsp;</span>","synthetic":false,"types":["actix_optional_middleware::Group"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()