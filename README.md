<div align="center">
  <h1>Actix Optional Middleware</h1>
  <p>
    <strong>Conditionally load middleware</strong>
  </p>

[![Documentation](https://img.shields.io/badge/docs-master-blue)](https://realaravinth.github.io/actix-optional-middleware/actix_optional_middleware/)
[![CI (Linux)](https://github.com/realaravinth/actix-optional-middleware/actions/workflows/linux.yml/badge.svg)](https://github.com/realaravinth/actix-optional-middleware/actions/workflows/linux.yml)
[![dependency status](https://deps.rs/repo/github/realaravinth/actix-optional-middleware/status.svg)](https://deps.rs/repo/github/realaravinth/actix-optional-middleware)
<br />

[![codecov](https://codecov.io/gh/realaravinth/actix-optional-middleware/branch/master/graph/badge.svg?token=TYZXLOOHYQ)](https://codecov.io/gh/realaravinth/actix-optional-middleware)

</div>

This library supports conditional loading of an Actix Web middleware.
When conditions are not met, a dummy middleware that simply forwards
requests are substituted.

## Usage

Add this to your `Cargo.toml`:

```toml
actix-optional-middleware = { version = "0.1", git = "https://github.com/realaravinth/actix-optional-middleware" }
```

## Example

```rust
use std::rc::Rc;

use actix_optional_middleware::{Group, Dummy};
use actix_web::dev::{AnyBody, Service, ServiceRequest,
ServiceResponse, Transform};
use actix_web::middleware::DefaultHeaders;
use actix_web::{web, App, Error, HttpServer, Responder, get};

#[get("/test", wrap = "get_group_middleware()")]
async fn h1() -> impl Responder {
    "Handler 1"
}

// flip this value to see dummy in action
const ACTIVE: bool = true;

fn get_group_middleware<S>() -> Group<Dummy, DefaultHeaders, S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error>
		+ 'static,
{
    if ACTIVE {
        Group::Real(Rc::new(DefaultHeaders::new()
               .header("Permissions-Policy", "interest-cohort=()"
        )))
    } else {
        Group::default()
    }
}
```
